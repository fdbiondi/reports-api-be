use actix_web::{get, HttpResponse};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[get("/health")]
pub async fn get_health() -> HttpResponse {
    HttpResponse::Ok().json(HealthResponse { status: "ok" })
}
