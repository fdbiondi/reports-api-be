mod api;

use api::reports::{create_report, get_report};

use actix_web::{middleware::Logger, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(get_report)
            .service(create_report)
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
