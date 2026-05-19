use crate::model::db::{
    begin_immediate_transaction, commit_transaction, open_connection, rollback_transaction,
};
use crate::model::nonce::{Nonce, NonceErr};
use crate::model::report::{Report, ReportErr};
use sqlite::Error as SqlError;

pub struct CreateReportInput {
    pub signature: String,
    pub title: String,
    pub description: String,
}

pub enum CreateReportResult {
    Created(Nonce),
    Retried(Nonce),
}

#[derive(Debug)]
pub enum CreateReportErr {
    Conflict {
        signature: String,
    },
    Db {
        operation: &'static str,
        resource: &'static str,
        signature: String,
    },
}

impl CreateReportErr {
    fn db(operation: &'static str, resource: &'static str, signature: &str) -> Self {
        Self::Db {
            operation,
            resource,
            signature: signature.to_string(),
        }
    }
}

fn ensure_nonce_for_retry(
    conn: &sqlite::Connection,
    signature: &str,
) -> Result<Nonce, CreateReportErr> {
    match Nonce::find_in_connection(conn, signature) {
        Ok(nonce) => Ok(nonce),
        // Retry path: prior attempt may have inserted report but failed before nonce write.
        Err(NonceErr::NotFound(_)) => Nonce::create_in_connection(conn, signature.to_string())
            .map_err(|_| CreateReportErr::db("repair_nonce", "nonce", signature)),
        Err(NonceErr::DbErr(_)) => Err(CreateReportErr::db("fetch", "nonce", signature)),
    }
}

fn persist_report_flow(
    conn: &sqlite::Connection,
    input: &CreateReportInput,
) -> Result<CreateReportResult, CreateReportErr> {
    match Report::find_in_connection(conn, &input.signature) {
        Ok(existing_report) => {
            if !existing_report.matches_payload(&input.title, &input.description) {
                Err(CreateReportErr::Conflict {
                    signature: input.signature.clone(),
                })
            } else {
                ensure_nonce_for_retry(conn, &input.signature).map(CreateReportResult::Retried)
            }
        }
        Err(ReportErr::NotFound(_)) => {
            if Report::create_in_connection(
                conn,
                input.signature.clone(),
                input.title.clone(),
                input.description.clone(),
            )
            .is_err()
            {
                Err(CreateReportErr::db("insert", "report", &input.signature))
            } else {
                match Nonce::find_in_connection(conn, &input.signature) {
                    Ok(nonce) => nonce
                        .increment_in_connection(conn)
                        .map(CreateReportResult::Created)
                        .map_err(|_| CreateReportErr::db("update", "nonce", &input.signature)),
                    Err(NonceErr::NotFound(_)) => {
                        Nonce::create_in_connection(conn, input.signature.clone())
                            .map(CreateReportResult::Created)
                            .map_err(|_| CreateReportErr::db("insert", "nonce", &input.signature))
                    }
                    Err(NonceErr::DbErr(_)) => {
                        Err(CreateReportErr::db("fetch", "nonce", &input.signature))
                    }
                }
            }
        }
        Err(ReportErr::DbErr(_)) => Err(CreateReportErr::db("fetch", "report", &input.signature)),
    }
}

pub fn create_or_retry(input: CreateReportInput) -> Result<CreateReportResult, CreateReportErr> {
    let conn = open_connection()
        .map_err(|_: SqlError| CreateReportErr::db("open", "report", &input.signature))?;

    begin_immediate_transaction(&conn).map_err(|_: SqlError| {
        CreateReportErr::db("begin_transaction", "report", &input.signature)
    })?;

    let outcome = persist_report_flow(&conn, &input);

    match outcome {
        Ok(result) => {
            if commit_transaction(&conn).is_err() {
                rollback_transaction(&conn);
                return Err(CreateReportErr::db(
                    "commit_transaction",
                    "report",
                    &input.signature,
                ));
            }

            Ok(result)
        }
        Err(err) => {
            rollback_transaction(&conn);
            Err(err)
        }
    }
}
