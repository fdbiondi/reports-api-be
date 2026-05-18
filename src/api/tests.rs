use super::configure;
use crate::test_support::env_lock;
use actix_web::{http::StatusCode, test, App};
use serde::Deserialize;
use sqlite::Connection;
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn seed_report_without_nonce(db_path: &str, signature: &str, title: &str, description: &str) {
    let conn = sqlite::open(db_path).unwrap();
    create_schema(&conn);

    conn.execute(format!(
        "INSERT INTO reports (uuid, signature, description, title, state)
         VALUES ('report-uuid', '{signature}', '{description}', '{title}', 'InProgress');"
    ))
    .unwrap();
}

fn create_empty_db(db_path: &str) {
    let conn = sqlite::open(db_path).unwrap();
    create_schema(&conn);
}

fn create_reports_only_db(db_path: &str) {
    let conn = sqlite::open(db_path).unwrap();
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
}

#[derive(Deserialize)]
struct ErrorDetailBody {
    field: String,
    issue: String,
}

#[derive(Deserialize)]
struct ErrorBody {
    code: String,
    error: String,
    details: Option<Vec<ErrorDetailBody>>,
}

#[actix_web::test]
async fn get_report_returns_ok_for_existing_report() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("get-report");
    seed_report_and_nonce(&db_path, "sig-123");
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::get()
        .uri("/reports/sig-123")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn get_health_returns_ok() {
    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn get_report_returns_not_found_for_missing_report() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("get-report-missing");
    create_empty_db(&db_path);
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::get()
        .uri("/reports/does-not-exist")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body: ErrorBody = test::read_body_json(resp).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.code, "NOT_FOUND");
    assert_eq!(body.error, "Report Not found!");
    let details = body.details.expect("not found details missing");
    assert_eq!(details.len(), 2);
    assert_eq!(details[0].field, "resource");
    assert_eq!(details[0].issue, "report");
    assert_eq!(details[1].field, "signature");
    assert_eq!(details[1].issue, "does-not-exist");
}

#[actix_web::test]
async fn get_nonce_returns_ok_for_existing_nonce() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("get-nonce");
    seed_report_and_nonce(&db_path, "sig-456");
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::get().uri("/nonces/sig-456").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn get_nonce_returns_not_found_for_missing_nonce() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("get-nonce-missing");
    create_empty_db(&db_path);
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::get()
        .uri("/nonces/does-not-exist")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body: ErrorBody = test::read_body_json(resp).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body.code, "NOT_FOUND");
    assert_eq!(body.error, "Nonce not found!");
    let details = body.details.expect("not found details missing");
    assert_eq!(details.len(), 2);
    assert_eq!(details[0].field, "resource");
    assert_eq!(details[0].issue, "nonce");
    assert_eq!(details[1].field, "signature");
    assert_eq!(details[1].issue, "does-not-exist");
}

#[actix_web::test]
async fn create_report_creates_report_and_nonce() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("create-report");
    create_empty_db(&db_path);
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
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

#[actix_web::test]
async fn create_report_returns_conflict_when_signature_already_exists() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("create-report-conflict");
    create_empty_db(&db_path);
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;

    let first_req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "signature": "sig-dup",
                "title": "First",
                "description": "first insert"
            }"#,
        )
        .to_request();
    let first_resp = test::call_service(&app, first_req).await;
    assert_eq!(first_resp.status(), StatusCode::CREATED);

    let second_req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "signature": "sig-dup",
                "title": "Second",
                "description": "duplicate insert"
            }"#,
        )
        .to_request();
    let second_resp = test::call_service(&app, second_req).await;
    let status = second_resp.status();
    let body: ErrorBody = test::read_body_json(second_resp).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body.code, "CONFLICT");
    assert_eq!(body.error, "Report already exists for this signature");
    let details = body.details.expect("conflict details missing");
    assert_eq!(details.len(), 2);
    assert_eq!(details[0].field, "resource");
    assert_eq!(details[0].issue, "report");
    assert_eq!(details[1].field, "signature");
    assert_eq!(details[1].issue, "sig-dup");
}

#[actix_web::test]
async fn create_report_retry_returns_ok_without_incrementing_nonce() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("create-report-retry");
    create_empty_db(&db_path);
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let payload = r#"{
        "signature": "sig-retry",
        "title": "Retry title",
        "description": "Retry description body"
    }"#;

    let first_req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(payload)
        .to_request();
    let first_resp = test::call_service(&app, first_req).await;
    assert_eq!(first_resp.status(), StatusCode::CREATED);

    let second_req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(payload)
        .to_request();
    let second_resp = test::call_service(&app, second_req).await;

    assert_eq!(second_resp.status(), StatusCode::OK);

    let conn = sqlite::open(&db_path).unwrap();
    conn.iterate(
        "SELECT COUNT(*) AS count, MAX(nonce) AS nonce FROM nonces WHERE signature = 'sig-retry';",
        |pairs| {
            assert_eq!(pairs[0].1.unwrap(), "1");
            assert_eq!(pairs[1].1.unwrap(), "1");
            true
        },
    )
    .unwrap();
}

#[actix_web::test]
async fn create_report_retry_repairs_missing_nonce_for_same_payload() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("create-report-retry-repair");
    seed_report_without_nonce(
        &db_path,
        "sig-repair",
        "Repair title",
        "Repair description body",
    );
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "signature": "sig-repair",
                "title": "Repair title",
                "description": "Repair description body"
            }"#,
        )
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    let conn = sqlite::open(&db_path).unwrap();
    conn.iterate(
        "SELECT nonce FROM nonces WHERE signature = 'sig-repair';",
        |pairs| {
            assert_eq!(pairs[0].1.unwrap(), "1");
            true
        },
    )
    .unwrap();
}

#[actix_web::test]
async fn create_report_returns_bad_request_for_invalid_payload() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("create-report-invalid-payload");
    create_empty_db(&db_path);
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "signature": "sig-invalid"
            }"#,
        )
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body: ErrorBody = test::read_body_json(resp).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body.code, "INVALID_JSON");
    assert!(body.error.starts_with("Invalid JSON payload:"));
    let details = body.details.expect("json details missing");
    assert_eq!(details.len(), 1);
    assert_eq!(details[0].field, "body");
    assert_eq!(details[0].issue, "invalid JSON payload");
}

#[actix_web::test]
async fn create_report_returns_validation_error_for_invalid_business_rules() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("create-report-validation-error");
    create_empty_db(&db_path);
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "signature": "   ",
                "title": "Ok title",
                "description": "This description has enough length"
            }"#,
        )
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body: ErrorBody = test::read_body_json(resp).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body.code, "VALIDATION_ERROR");
    assert_eq!(body.error, "Validation failed");
    let details = body.details.expect("validation details missing");
    assert_eq!(details.len(), 1);
    assert_eq!(details[0].field, "signature");
    assert_eq!(details[0].issue, "cannot be empty");
}

#[actix_web::test]
async fn create_report_normalizes_title_and_description() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("create-report-normalization");
    create_empty_db(&db_path);
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "signature": "  sig-normalized  ",
                "title": "  A    normalized   title  ",
                "description": "  This    description     should be normalized   "
            }"#,
        )
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CREATED);

    let conn = sqlite::open(&db_path).unwrap();
    let mut title = String::new();
    let mut description = String::new();
    conn.iterate(
        "SELECT title, description FROM reports WHERE signature = 'sig-normalized';",
        |pairs| {
            title = pairs[0].1.unwrap_or_default().to_string();
            description = pairs[1].1.unwrap_or_default().to_string();
            true
        },
    )
    .unwrap();

    assert_eq!(title, "A normalized title");
    assert_eq!(description, "This description should be normalized");
}

#[actix_web::test]
async fn create_report_returns_internal_server_error_for_invalid_db_path() {
    let _guard = env_lock().lock().unwrap();
    env::set_var("DB_PATH", "/tmp/reports-api-missing-dir/data.db");

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "signature": "sig-db-error",
                "title": "Will fail",
                "description": "db path is invalid"
            }"#,
        )
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body: ErrorBody = test::read_body_json(resp).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body.code, "INTERNAL_ERROR");
    assert_eq!(body.error, "Database operation failed");
    let details = body.details.expect("internal details missing");
    assert_eq!(details.len(), 3);
    assert_eq!(details[0].field, "operation");
    assert_eq!(details[0].issue, "open");
    assert_eq!(details[1].field, "resource");
    assert_eq!(details[1].issue, "report");
    assert_eq!(details[2].field, "signature");
    assert_eq!(details[2].issue, "sig-db-error");
}

#[actix_web::test]
async fn create_report_returns_internal_error_when_db_is_locked() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("create-report-db-locked");
    create_empty_db(&db_path);
    env::set_var("DB_PATH", &db_path);

    let lock_conn = sqlite::open(&db_path).unwrap();
    lock_conn.execute("BEGIN EXCLUSIVE;").unwrap();

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "signature": "sig-locked",
                "title": "Locked title",
                "description": "Locked description body"
            }"#,
        )
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body: ErrorBody = test::read_body_json(resp).await;

    lock_conn.execute("ROLLBACK;").unwrap();

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body.code, "INTERNAL_ERROR");
    assert_eq!(body.error, "Database operation failed");
    let details = body.details.expect("internal details missing");
    assert_eq!(details.len(), 3);
    assert_eq!(details[0].field, "operation");
    assert_eq!(details[0].issue, "begin_transaction");
    assert_eq!(details[1].field, "resource");
    assert_eq!(details[1].issue, "report");
    assert_eq!(details[2].field, "signature");
    assert_eq!(details[2].issue, "sig-locked");
}

#[actix_web::test]
async fn get_report_returns_internal_error_for_corrupt_db_file() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("get-report-corrupt-db");
    fs::write(&db_path, b"not-a-sqlite-database").unwrap();
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::get()
        .uri("/reports/corrupt-signature")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body: ErrorBody = test::read_body_json(resp).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body.code, "INTERNAL_ERROR");
    assert_eq!(body.error, "Database operation failed");
    let details = body.details.expect("internal details missing");
    assert_eq!(details.len(), 3);
    assert_eq!(details[0].field, "operation");
    assert_eq!(details[0].issue, "fetch");
    assert_eq!(details[1].field, "resource");
    assert_eq!(details[1].issue, "report");
    assert_eq!(details[2].field, "signature");
    assert_eq!(details[2].issue, "corrupt-signature");
}

#[actix_web::test]
async fn create_report_rolls_back_when_nonce_table_is_missing() {
    let _guard = env_lock().lock().unwrap();
    let db_path = temp_db_path("create-report-missing-nonce-table");
    create_reports_only_db(&db_path);
    env::set_var("DB_PATH", &db_path);

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::post()
        .uri("/reports")
        .insert_header(("Content-Type", "application/json"))
        .set_payload(
            r#"{
                "signature": "sig-missing-nonce-table",
                "title": "Schema failure",
                "description": "Nonce table is missing in this database"
            }"#,
        )
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body: ErrorBody = test::read_body_json(resp).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body.code, "INTERNAL_ERROR");
    assert_eq!(body.error, "Database operation failed");
    let details = body.details.expect("internal details missing");
    assert_eq!(details.len(), 3);
    assert_eq!(details[0].field, "operation");
    assert_eq!(details[0].issue, "fetch");
    assert_eq!(details[1].field, "resource");
    assert_eq!(details[1].issue, "nonce");
    assert_eq!(details[2].field, "signature");
    assert_eq!(details[2].issue, "sig-missing-nonce-table");

    let conn = sqlite::open(&db_path).unwrap();
    conn.iterate(
        "SELECT COUNT(*) AS count FROM reports WHERE signature = 'sig-missing-nonce-table';",
        |pairs| {
            assert_eq!(pairs[0].1.unwrap(), "0");
            true
        },
    )
    .unwrap();
}

#[actix_web::test]
async fn get_report_returns_internal_server_error_for_invalid_db_path() {
    let _guard = env_lock().lock().unwrap();
    env::set_var("DB_PATH", "/tmp/reports-api-missing-dir/data.db");

    let app = test::init_service(App::new().configure(configure)).await;
    let req = test::TestRequest::get()
        .uri("/reports/any-signature")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body: ErrorBody = test::read_body_json(resp).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body.code, "INTERNAL_ERROR");
    assert_eq!(body.error, "Database operation failed");
    let details = body.details.expect("internal details missing");
    assert_eq!(details.len(), 3);
    assert_eq!(details[0].field, "operation");
    assert_eq!(details[0].issue, "fetch");
    assert_eq!(details[1].field, "resource");
    assert_eq!(details[1].issue, "report");
    assert_eq!(details[2].field, "signature");
    assert_eq!(details[2].issue, "any-signature");
}
