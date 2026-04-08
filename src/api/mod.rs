pub mod nonces;
pub mod reports;

use actix_web::{
    error::InternalError,
    http::StatusCode,
    web::{self, JsonConfig},
    HttpResponse,
};
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
    cfg.app_data(JsonConfig::default().error_handler(|err, _req| {
        let response = error_response(
            StatusCode::BAD_REQUEST,
            format!("Invalid JSON payload: {err}"),
        );
        InternalError::from_response(err, response).into()
    }))
        .service(nonces::get_nonce)
        .service(reports::get_report)
        .service(reports::create_report);
}
