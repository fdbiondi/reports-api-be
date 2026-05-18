pub mod health;
pub mod nonces;
pub mod reports;
#[cfg(test)]
mod tests;

use crate::error::{ApiError, ApiErrorDetail};
use actix_web::{
    error::InternalError,
    web::{self, JsonConfig},
    ResponseError,
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.app_data(JsonConfig::default().error_handler(|err, _req| {
        let response = ApiError::invalid_json(format!("Invalid JSON payload: {err}"))
            .with_details(vec![ApiErrorDetail::new("body", "invalid JSON payload")])
            .error_response();
        InternalError::from_response(err, response).into()
    }))
    .service(health::get_health)
    .service(nonces::get_nonce)
    .service(reports::get_report)
    .service(reports::create_report);
}
