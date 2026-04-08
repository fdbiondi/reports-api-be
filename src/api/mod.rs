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
    code: String,
    error: String,
}

pub fn error_response(
    status: StatusCode,
    code: impl Into<String>,
    message: impl Into<String>,
) -> HttpResponse {
    HttpResponse::build(status).json(ErrorResponse {
        code: code.into(),
        error: message.into(),
    })
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.app_data(JsonConfig::default().error_handler(|err, _req| {
        let response = error_response(
            StatusCode::BAD_REQUEST,
            "INVALID_JSON",
            format!("Invalid JSON payload: {err}"),
        );
        InternalError::from_response(err, response).into()
    }))
        .service(nonces::get_nonce)
        .service(reports::get_report)
        .service(reports::create_report);
}
