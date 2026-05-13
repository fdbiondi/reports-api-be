use crate::api::error_response;
use crate::model::nonce::Nonce;
use crate::model::report::{Report, ReportErr};

use actix_web::{get, http::StatusCode, post, web, HttpResponse};
use serde::{Deserialize, Serialize};

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

#[post("/reports")]
pub async fn create_report(data: web::Json<PostReportRequest>) -> HttpResponse {
    let payload = match validate_and_normalize(&data) {
        Ok(payload) => payload,
        Err(message) => {
            return error_response(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message);
        }
    };

    // create report from data
    let report = Report::create(
        payload.signature.to_string(),
        payload.title.to_string(),
        payload.description.to_string(),
    );

    match report {
        Ok(report) => report,
        Err(ReportErr::NotFound(message)) => {
            return error_response(StatusCode::NOT_FOUND, "NOT_FOUND", message);
        }
        Err(ReportErr::DbErr(err)) => {
            let err_message = err.to_string();
            if err_message.contains("UNIQUE constraint failed") {
                return error_response(
                    StatusCode::CONFLICT,
                    "CONFLICT",
                    "Report already exists for this signature",
                );
            }
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Failed to create report",
            );
        }
    };

    // find nonce from signature
    let nonce = match Nonce::find(payload.signature.to_string()) {
        Ok(nonce) => match nonce.increment() {
            Ok(nonce) => nonce,
            Err(_) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Failed to update nonce",
                );
            }
        },
        // if not exists -> insert nonce
        Err(_) => match Nonce::create(payload.signature.to_string()) {
            Ok(nonce) => nonce,
            Err(_) => {
                return error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Failed to create nonce",
                );
            }
        },
    };

    // return nonce
    HttpResponse::Created().json(nonce)
}
