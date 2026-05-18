use crate::error::ApiError;
use crate::model::nonce::{Nonce, NonceErr};

use actix_web::{get, web, HttpResponse};

#[get("/nonces/{signature}")]
pub async fn get_nonce(signature: web::Path<String>) -> Result<HttpResponse, ApiError> {
    match Nonce::find(signature.to_string()) {
        Ok(nonce) => Ok(HttpResponse::Ok().json(nonce)),
        Err(NonceErr::NotFound(message)) => Err(ApiError::not_found(message)),
        Err(NonceErr::DbErr(_)) => Err(ApiError::internal("Failed to fetch nonce")),
    }
}
