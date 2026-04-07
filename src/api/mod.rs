pub mod nonces;
pub mod reports;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(nonces::get_nonce)
        .service(reports::get_report)
        .service(reports::create_report);
}
