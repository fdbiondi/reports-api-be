#[path = "../model/mod.rs"]
mod model;

use model::nonce::Nonce;

use actix_web::{get, web, HttpResponse};

#[get("/nonces/{signature}")]
pub async fn get_nonce(signature: web::Path<String>) -> HttpResponse {
    match Nonce::find(signature.to_string()) {
        Ok(nonce) => HttpResponse::Ok().json(nonce),
        Err(_) => HttpResponse::NotFound().json("Nonce not found!"),
    }
}
