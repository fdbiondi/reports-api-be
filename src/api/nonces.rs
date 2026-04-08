use crate::api::error_response;
use crate::model::nonce::Nonce;

use actix_web::{get, http::StatusCode, web, HttpResponse};

#[get("/nonces/{signature}")]
pub async fn get_nonce(signature: web::Path<String>) -> HttpResponse {
    match Nonce::find(signature.to_string()) {
        Ok(nonce) => HttpResponse::Ok().json(nonce),
        Err(_) => error_response(StatusCode::NOT_FOUND, "Nonce not found!"),
    }
}
