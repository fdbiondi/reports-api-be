mod api;

use api::nonces::get_nonce;
use api::reports::{create_report, get_report};

use actix_web::{middleware::Logger, App, HttpServer};
use dotenv::dotenv;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(get_nonce)
            .service(get_report)
            .service(create_report)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
