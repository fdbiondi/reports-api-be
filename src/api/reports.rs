#[path = "../model/mod.rs"]
mod model;

use actix_web::{get, post, web, HttpResponse, Responder, ResponseError, Result as AWResult};
use serde::{Deserialize, Serialize};

#[get("/reports/{signature}")]
pub async fn get_report(signature: web::Path<String>) -> AWResult<impl Responder> {
    // TODO

    // find report by signature -> SELECT * FROM reports where signature = ?

    // if fails -> return 404

    // if finded -> return report

    let report = model::report::Report::new(
        signature.to_string(),
        "title desc".to_string(),
        "test desc".to_string(),
    );

    Ok(web::Json(report))
}

#[derive(Serialize, Deserialize)]
pub struct PostReportRequest {
    nonce: String,
    signature: String,
    title: String,
    description: String,
}

#[post("/reports")]
pub async fn create_report(data: web::Json<PostReportRequest>) -> HttpResponse {
    // create report from data
    let report = model::report::Report::new(
        data.signature.to_string(),
        data.title.to_string(),
        data.description.to_string(),
    );

    // find nonce from signature -> SELECT * FROM nonces where signature = ?

    // if exists
    // update nonce -> UPDATE nonces SET nonce = ? WHERE signature = ?

    // if not exists
    // insert nonce

    // return nonce

    HttpResponse::Created().json(report)
}
