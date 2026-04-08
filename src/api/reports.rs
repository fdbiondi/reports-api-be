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

#[post("/reports")]
pub async fn create_report(data: web::Json<PostReportRequest>) -> HttpResponse {
    // create report from data
    let report = Report::create(
        data.signature.to_string(),
        data.title.to_string(),
        data.description.to_string(),
    );

    match report {
        Ok(report) => report,
        Err(ReportErr::NotFound(message)) => return error_response(StatusCode::NOT_FOUND, message),
        Err(ReportErr::DbErr(err)) => {
            let err_message = err.to_string();
            if err_message.contains("UNIQUE constraint failed") {
                return error_response(
                    StatusCode::CONFLICT,
                    "Report already exists for this signature",
                );
            }
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create report");
        }
    };

    // find nonce from signature
    let nonce = match Nonce::find(data.signature.to_string()) {
        Ok(nonce) => match nonce.increment() {
            Ok(nonce) => nonce,
            Err(_) => {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to update nonce");
            }
        },
        // if not exists -> insert nonce
        Err(_) => match Nonce::create(data.signature.to_string()) {
            Ok(nonce) => nonce,
            Err(_) => {
                return error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create nonce");
            }
        },
    };

    // return nonce
    HttpResponse::Created().json(nonce)
}
