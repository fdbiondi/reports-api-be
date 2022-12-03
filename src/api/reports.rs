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

    match report {
        Ok(report) => report,
        Err(_) => return HttpResponse::NotFound().json("Failed to create report"),
    };

    // find nonce from signature
    let nonce = match Nonce::find(data.nonce.to_string()) {
        // if exists -> update nonce
        Ok(nonce) => match nonce.increment() {
            Ok(nonce) => nonce,
            Err(_) => return HttpResponse::NotFound().json("Failed to update nonce"),
        },
        // if not exists -> insert nonce
        Err(_) => match Nonce::create(data.nonce.to_string()) {
            Ok(nonce) => nonce,
            Err(_) => return HttpResponse::NotFound().json("Failed to create nonce"),
        },
    };

    // return nonce
    HttpResponse::Created().json(nonce)
}
