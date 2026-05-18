use crate::api::error_response;
use crate::model::db::open_connection;
use crate::model::nonce::{Nonce, NonceErr};
use crate::model::report::{Report, ReportErr};

use actix_web::{get, http::StatusCode, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlite::Connection;

#[get("/reports/{signature}")]
pub async fn get_report(signature: web::Path<String>) -> Result<HttpResponse, ReportErr> {
    match Report::find(signature.to_string()) {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(err) => Err(err),
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

fn validate_and_normalize(payload: &PostReportRequest) -> Result<ValidatedReportInput, String> {
    let signature = payload.signature.trim().to_string();
    let title = normalize_whitespace(&payload.title);
    let description = normalize_whitespace(&payload.description);

    let signature_len = signature.chars().count();
    if signature_len == 0 {
        return Err("Field 'signature' cannot be empty".to_string());
    }
    if signature_len > 132 {
        return Err("Field 'signature' must be at most 132 characters".to_string());
    }

    let title_len = title.chars().count();
    if title_len < 3 {
        return Err("Field 'title' must be at least 3 characters".to_string());
    }
    if title_len > 50 {
        return Err("Field 'title' must be at most 50 characters".to_string());
    }

    let description_len = description.chars().count();
    if description_len < 10 {
        return Err("Field 'description' must be at least 10 characters".to_string());
    }
    if description_len > 5000 {
        return Err("Field 'description' must be at most 5000 characters".to_string());
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

fn ensure_nonce_for_retry(conn: &Connection, signature: &str) -> Result<Nonce, HttpResponse> {
    match Nonce::find_in_connection(conn, signature) {
        Ok(nonce) => Ok(nonce),
        Err(NonceErr::NotFound(_)) => Nonce::create_in_connection(conn, signature.to_string())
            .map_err(|_| {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Failed to repair nonce for retried report",
                )
            }),
        Err(NonceErr::DbErr(_)) => Err(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "Failed to fetch nonce",
        )),
    }
}

#[post("/reports")]
pub async fn create_report(data: web::Json<PostReportRequest>) -> HttpResponse {
    let payload = match validate_and_normalize(&data) {
        Ok(payload) => payload,
        Err(message) => {
            return error_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message);
        }
    };

    let conn = match open_connection() {
        Ok(conn) => conn,
        Err(_) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Failed to create report",
            );
        }
    };

    if conn.execute("BEGIN IMMEDIATE TRANSACTION;").is_err() {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "Failed to start transaction",
        );
    }

    let outcome = match Report::find_in_connection(&conn, &payload.signature) {
        Ok(existing_report) => {
            if !existing_report.matches_payload(&payload.title, &payload.description) {
                Err(error_response(
                    StatusCode::CONFLICT,
                    "CONFLICT",
                    "Report already exists for this signature",
                ))
            } else {
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
                Err(error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Failed to create report",
                ))
            } else {
                match Nonce::find_in_connection(&conn, &payload.signature) {
                    Ok(nonce) => nonce.increment_in_connection(&conn).map_err(|_| {
                        error_response(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "INTERNAL_ERROR",
                            "Failed to update nonce",
                        )
                    }),
                    Err(NonceErr::NotFound(_)) => {
                        Nonce::create_in_connection(&conn, payload.signature.to_string()).map_err(
                            |_| {
                                error_response(
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    "INTERNAL_ERROR",
                                    "Failed to create nonce",
                                )
                            },
                        )
                    }
                    Err(NonceErr::DbErr(_)) => Err(error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        "Failed to fetch nonce",
                    )),
                }
                .map(|nonce| (StatusCode::CREATED, nonce))
            }
        }
        Err(ReportErr::DbErr(_)) => Err(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "Failed to create report",
        )),
    };

    match outcome {
        Ok((status, nonce)) => {
            if conn.execute("COMMIT;").is_err() {
                rollback_transaction(&conn);
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Failed to commit transaction",
                );
            }

            HttpResponse::build(status).json(nonce)
        }
        Err(response) => {
            rollback_transaction(&conn);
            response
        }
    }
}
