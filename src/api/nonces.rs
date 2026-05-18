use crate::api::error_response;
use crate::model::nonce::{Nonce, NonceErr};

use actix_web::{get, http::StatusCode, web, HttpResponse};

#[get("/nonces/{signature}")]
pub async fn get_nonce(signature: web::Path<String>) -> HttpResponse {
    match Nonce::find(signature.to_string()) {
        Ok(nonce) => HttpResponse::Ok().json(nonce),
        Err(NonceErr::NotFound(message)) => {
            error_response(StatusCode::NOT_FOUND, "NOT_FOUND", message)
        }
        Err(NonceErr::DbErr(_)) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "Failed to fetch nonce",
        ),
    }
}
