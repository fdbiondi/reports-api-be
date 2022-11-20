#[path = "../model/mod.rs"]
mod model;

use model::nonce::Nonce;
use model::report::{Report, ReportErr};

use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};

#[get("/reports/{signature}")]
pub async fn get_report(signature: web::Path<String>) -> Result<HttpResponse, ReportErr> {
    match Report::find(signature.to_string()) {
        Ok(res) => Ok(HttpResponse::Created().json(res)),
        Err(err) => Err(err), // TODO if fails -> return 404
    }
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
    let report = Report::create(
        data.signature.to_string(),
        data.title.to_string(),
        data.description.to_string(),
    );

    // find nonce from signature -> SELECT * FROM nonces where signature = ?

    let mut nonce = Nonce::find(data.nonce.to_string());

    if Some(nonce) {
        // if exists -> update nonce -> UPDATE nonces SET nonce = ? WHERE signature = ?
        nonce.update();
    } else {
        // if not exists -> insert nonce
        nonce = Nonce::create(data.nonce.to_string());
    }

    /* match Nonce::find(data.nonce.to_string()) {
        Some(nonce) => nonce.update(),
        None => Nonce::create(data.nonce.to_string()),
    } */

    // return nonce
    HttpResponse::Created().json(nonce)
}
