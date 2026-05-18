use crate::error::{ApiError, ApiErrorDetail};
use crate::model::db::open_connection;
use crate::model::nonce::{Nonce, NonceErr};
use crate::model::report::{Report, ReportErr};

use actix_web::{get, http::StatusCode, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlite::Connection;

#[get("/reports/{signature}")]
pub async fn get_report(signature: web::Path<String>) -> Result<HttpResponse, ApiError> {
    match Report::find(signature.to_string()) {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(ReportErr::NotFound(message)) => Err(ApiError::not_found(message).with_details(vec![
            ApiErrorDetail::new("resource", "report"),
            ApiErrorDetail::new("signature", signature.as_str()),
        ])),
        Err(ReportErr::DbErr(_)) => Err(ApiError::db_failure("fetch", "report")
            .with_details(vec![ApiErrorDetail::new("signature", signature.as_str())])),
    }
}

#[derive(Serialize, Deserialize)]
pub struct PostReportRequest {
    signature: String,
    title: String,
    description: String,
}

struct ValidatedReportInput {
    signature: String,
    title: String,
    description: String,
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn validate_and_normalize(payload: &PostReportRequest) -> Result<ValidatedReportInput, ApiError> {
    // Persist and compare normalized values so retries are judged on canonical payload,
    // not on caller-specific spacing.
    let signature = payload.signature.trim().to_string();
    let title = normalize_whitespace(&payload.title);
    let description = normalize_whitespace(&payload.description);

    let signature_len = signature.chars().count();
    if signature_len == 0 {
        return Err(ApiError::validation("Validation failed")
            .with_details(vec![ApiErrorDetail::new("signature", "cannot be empty")]));
    }
    if signature_len > 132 {
        return Err(ApiError::validation("Validation failed").with_details(vec![
            ApiErrorDetail::new("signature", "must be at most 132 characters"),
        ]));
    }

    let title_len = title.chars().count();
    if title_len < 3 {
        return Err(ApiError::validation("Validation failed").with_details(vec![
            ApiErrorDetail::new("title", "must be at least 3 characters"),
        ]));
    }
    if title_len > 50 {
        return Err(ApiError::validation("Validation failed").with_details(vec![
            ApiErrorDetail::new("title", "must be at most 50 characters"),
        ]));
    }

    let description_len = description.chars().count();
    if description_len < 10 {
        return Err(ApiError::validation("Validation failed").with_details(vec![
            ApiErrorDetail::new("description", "must be at least 10 characters"),
        ]));
    }
    if description_len > 5000 {
        return Err(ApiError::validation("Validation failed").with_details(vec![
            ApiErrorDetail::new("description", "must be at most 5000 characters"),
        ]));
    }

    Ok(ValidatedReportInput {
        signature,
        title,
        description,
    })
}

fn rollback_transaction(conn: &Connection) {
    let _ = conn.execute("ROLLBACK;");
}

fn ensure_nonce_for_retry(conn: &Connection, signature: &str) -> Result<Nonce, ApiError> {
    match Nonce::find_in_connection(conn, signature) {
        Ok(nonce) => Ok(nonce),
        // Retry path: prior attempt may have inserted report but failed before nonce write.
        Err(NonceErr::NotFound(_)) => Nonce::create_in_connection(conn, signature.to_string())
            .map_err(|_| {
                ApiError::internal("Failed to repair nonce for retried report").with_details(vec![
                    ApiErrorDetail::new("operation", "repair_nonce"),
                    ApiErrorDetail::new("signature", signature),
                ])
            }),
        Err(NonceErr::DbErr(_)) => Err(ApiError::db_failure("fetch", "nonce")
            .with_details(vec![ApiErrorDetail::new("signature", signature)])),
    }
}

#[post("/reports")]
pub async fn create_report(data: web::Json<PostReportRequest>) -> Result<HttpResponse, ApiError> {
    let payload = match validate_and_normalize(&data) {
        Ok(payload) => payload,
        Err(err) => return Err(err),
    };

    let conn = match open_connection() {
        Ok(conn) => conn,
        Err(_) => {
            return Err(ApiError::db_failure("open", "report")
                .with_details(vec![ApiErrorDetail::new("signature", &payload.signature)]));
        }
    };

    if conn.execute("BEGIN IMMEDIATE TRANSACTION;").is_err() {
        return Err(ApiError::db_failure("begin_transaction", "report")
            .with_details(vec![ApiErrorDetail::new("signature", &payload.signature)]));
    }

    let outcome = match Report::find_in_connection(&conn, &payload.signature) {
        Ok(existing_report) => {
            // Same signature + different business payload is conflict. Same normalized payload
            // is treated as idempotent retry and must not mutate nonce again.
            if !existing_report.matches_payload(&payload.title, &payload.description) {
                Err(
                    ApiError::conflict("Report already exists for this signature").with_details(
                        vec![
                            ApiErrorDetail::new("resource", "report"),
                            ApiErrorDetail::new("signature", &payload.signature),
                        ],
                    ),
                )
            } else {
                // Same normalized payload counts as safe retry, not duplicate mutation.
                ensure_nonce_for_retry(&conn, &payload.signature)
                    .map(|nonce| (StatusCode::OK, nonce))
            }
        }
        Err(ReportErr::NotFound(_)) => {
            if Report::create_in_connection(
                &conn,
                payload.signature.to_string(),
                payload.title.to_string(),
                payload.description.to_string(),
            )
            .is_err()
            {
                Err(ApiError::db_failure("insert", "report")
                    .with_details(vec![ApiErrorDetail::new("signature", &payload.signature)]))
            } else {
                match Nonce::find_in_connection(&conn, &payload.signature) {
                    Ok(nonce) => nonce.increment_in_connection(&conn).map_err(|_| {
                        ApiError::db_failure("update", "nonce").with_details(vec![
                            ApiErrorDetail::new("signature", &payload.signature),
                        ])
                    }),
                    Err(NonceErr::NotFound(_)) => {
                        Nonce::create_in_connection(&conn, payload.signature.to_string()).map_err(
                            |_| {
                                ApiError::db_failure("insert", "nonce").with_details(vec![
                                    ApiErrorDetail::new("signature", &payload.signature),
                                ])
                            },
                        )
                    }
                    Err(NonceErr::DbErr(_)) => Err(ApiError::db_failure("fetch", "nonce")
                        .with_details(vec![ApiErrorDetail::new("signature", &payload.signature)])),
                }
                .map(|nonce| (StatusCode::CREATED, nonce))
            }
        }
        Err(ReportErr::DbErr(_)) => Err(ApiError::db_failure("fetch", "report")
            .with_details(vec![ApiErrorDetail::new("signature", &payload.signature)])),
    };

    match outcome {
        Ok((status, nonce)) => {
            if conn.execute("COMMIT;").is_err() {
                rollback_transaction(&conn);
                return Err(ApiError::db_failure("commit_transaction", "report")
                    .with_details(vec![ApiErrorDetail::new("signature", &payload.signature)]));
            }

            Ok(HttpResponse::build(status).json(nonce))
        }
        Err(response) => {
            rollback_transaction(&conn);
            Err(response)
        }
    }
}
