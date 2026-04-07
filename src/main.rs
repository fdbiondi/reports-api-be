mod api;

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
            .configure(api::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test};
    use sqlite::Connection;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn temp_db_path(test_name: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        env::temp_dir()
            .join(format!("reports-api-{test_name}-{nanos}.db"))
            .to_string_lossy()
            .into_owned()
    }

    fn create_schema(conn: &Connection) {
        conn.execute(
            "CREATE TABLE reports (
                uuid NVARCHAR(36) UNIQUE NOT NULL,
                signature NVARCHAR(132) PRIMARY KEY NOT NULL,
                description TEXT NOT NULL,
                title NVARCHAR(50) NOT NULL,
                state NVARCHAR(12) NOT NULL
            );",
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE nonces (
                uuid NVARCHAR(36) UNIQUE NOT NULL,
                signature NVARCHAR(132) PRIMARY KEY NOT NULL,
                nonce INTEGER NOT NULL
            );",
        )
        .unwrap();
    }

    fn seed_report_and_nonce(db_path: &str, signature: &str) {
        let conn = sqlite::open(db_path).unwrap();
        create_schema(&conn);

        conn.execute(format!(
            "INSERT INTO reports (uuid, signature, description, title, state)
             VALUES ('report-uuid', '{signature}', 'desc', 'title', 'InProgress');"
        ))
        .unwrap();

        conn.execute(format!(
            "INSERT INTO nonces (uuid, signature, nonce)
             VALUES ('nonce-uuid', '{signature}', 3);"
        ))
        .unwrap();
    }

    fn create_empty_db(db_path: &str) {
        let conn = sqlite::open(db_path).unwrap();
        create_schema(&conn);
    }

    #[actix_web::test]
    async fn get_report_returns_ok_for_existing_report() {
        let _guard = env_lock().lock().unwrap();
        let db_path = temp_db_path("get-report");
        seed_report_and_nonce(&db_path, "sig-123");
        env::set_var("DB_PATH", &db_path);

        let app = test::init_service(App::new().configure(api::configure)).await;
        let req = test::TestRequest::get()
            .uri("/reports/sig-123")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn get_nonce_returns_ok_for_existing_nonce() {
        let _guard = env_lock().lock().unwrap();
        let db_path = temp_db_path("get-nonce");
        seed_report_and_nonce(&db_path, "sig-456");
        env::set_var("DB_PATH", &db_path);

        let app = test::init_service(App::new().configure(api::configure)).await;
        let req = test::TestRequest::get()
            .uri("/nonces/sig-456")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn create_report_creates_report_and_nonce() {
        let _guard = env_lock().lock().unwrap();
        let db_path = temp_db_path("create-report");
        create_empty_db(&db_path);
        env::set_var("DB_PATH", &db_path);

        let app = test::init_service(App::new().configure(api::configure)).await;
        let req = test::TestRequest::post()
            .uri("/reports")
            .insert_header(("Content-Type", "application/json"))
            .set_payload(
                r#"{
                    "signature": "sig-new",
                    "title": "Created from test",
                    "description": "integration-style endpoint test"
                }"#,
            )
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED);

        let conn = sqlite::open(&db_path).unwrap();
        let report_count: i64 = conn
            .iterate(
                "SELECT COUNT(*) AS count FROM reports WHERE signature = 'sig-new';",
                |pairs| {
                    assert_eq!(pairs[0].1.unwrap(), "1");
                    true
                },
            )
            .map(|_| 1_i64)
            .unwrap();
        let nonce_count: i64 = conn
            .iterate(
                "SELECT COUNT(*) AS count FROM nonces WHERE signature = 'sig-new' AND nonce = 1;",
                |pairs| {
                    assert_eq!(pairs[0].1.unwrap(), "1");
                    true
                },
            )
            .map(|_| 1_i64)
            .unwrap();

        assert_eq!(report_count, 1);
        assert_eq!(nonce_count, 1);
    }
}
