pub mod nonces;
pub mod reports;

use actix_web::{http::StatusCode, web, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn error_response(status: StatusCode, message: impl Into<String>) -> HttpResponse {
    HttpResponse::build(status).json(ErrorResponse {
        error: message.into(),
    })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(nonces::get_nonce)
        .service(reports::get_report)
        .service(reports::create_report);
}
