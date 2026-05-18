use crate::error::{ApiError, ApiErrorDetail};
use crate::model::nonce::{Nonce, NonceErr};

use actix_web::{get, web, HttpResponse};

#[get("/nonces/{signature}")]
pub async fn get_nonce(signature: web::Path<String>) -> Result<HttpResponse, ApiError> {
    match Nonce::find(signature.to_string()) {
        Ok(nonce) => Ok(HttpResponse::Ok().json(nonce)),
        Err(NonceErr::NotFound(message)) => Err(ApiError::not_found(message).with_details(vec![
            ApiErrorDetail::new("resource", "nonce"),
            ApiErrorDetail::new("signature", signature.as_str()),
        ])),
        Err(NonceErr::DbErr(_)) => Err(ApiError::db_failure("fetch", "nonce")
            .with_details(vec![ApiErrorDetail::new("signature", signature.as_str())])),
    }
}
