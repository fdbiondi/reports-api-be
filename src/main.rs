mod api;

use api::reports::{
    get_report
};

use actix_web::{get, App, HttpResponse, HttpServer, Responder, middleware::Logger};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json("Hello from rust and reporter!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    HttpServer::new(|| 
            App::new()
            .wrap(Logger::default())
            .service(get_report)
        )
        .bind(("0.0.0.0", 80))?
        .run()
        .await
}
