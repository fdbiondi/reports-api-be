use crate::error::{ApiError, ApiErrorDetail};
use crate::model::report::{Report, ReportErr};
use crate::model::report_submission::{create_or_retry, CreateReportInput, CreateReportResult};

use actix_web::{get, http::StatusCode, post, web, HttpResponse};
use serde::{Deserialize, Serialize};

#[get("/reports/{signature}")]
pub async fn get_report(signature: web::Path<String>) -> Result<HttpResponse, ApiError> {
    match Report::find(signature.to_string()) {
        Ok(res) => Ok(HttpResponse::Ok().json(res)),
        Err(ReportErr::NotFound(message)) => Err(ApiError::not_found(message).with_details(vec![
            ApiErrorDetail::new("resource", "report"),
            ApiErrorDetail::new("signature", signature.as_str()),
        ])),
        Err(ReportErr::DbErr(_)) => Err(ApiError::db_failure("fetch", "report")
            .with_details(vec![ApiErrorDetail::new("signature", signature.as_str())])),
    }
}

#[derive(Serialize, Deserialize)]
pub struct PostReportRequest {
    signature: String,
    title: String,
    description: String,
}

struct ValidatedReportInput {
    signature: String,
    title: String,
    description: String,
}

fn normalize_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn validate_and_normalize(payload: &PostReportRequest) -> Result<ValidatedReportInput, ApiError> {
    // Persist and compare normalized values so retries are judged on canonical payload,
    // not on caller-specific spacing.
    let signature = payload.signature.trim().to_string();
    let title = normalize_whitespace(&payload.title);
    let description = normalize_whitespace(&payload.description);

    let signature_len = signature.chars().count();
    if signature_len == 0 {
        return Err(ApiError::validation("Validation failed")
            .with_details(vec![ApiErrorDetail::new("signature", "cannot be empty")]));
    }
    if signature_len > 132 {
        return Err(ApiError::validation("Validation failed").with_details(vec![
            ApiErrorDetail::new("signature", "must be at most 132 characters"),
        ]));
    }

    let title_len = title.chars().count();
    if title_len < 3 {
        return Err(ApiError::validation("Validation failed").with_details(vec![
            ApiErrorDetail::new("title", "must be at least 3 characters"),
        ]));
    }
    if title_len > 50 {
        return Err(ApiError::validation("Validation failed").with_details(vec![
            ApiErrorDetail::new("title", "must be at most 50 characters"),
        ]));
    }

    let description_len = description.chars().count();
    if description_len < 10 {
        return Err(ApiError::validation("Validation failed").with_details(vec![
            ApiErrorDetail::new("description", "must be at least 10 characters"),
        ]));
    }
    if description_len > 5000 {
        return Err(ApiError::validation("Validation failed").with_details(vec![
            ApiErrorDetail::new("description", "must be at most 5000 characters"),
        ]));
    }

    Ok(ValidatedReportInput {
        signature,
        title,
        description,
    })
}

#[post("/reports")]
pub async fn create_report(data: web::Json<PostReportRequest>) -> Result<HttpResponse, ApiError> {
    let payload = match validate_and_normalize(&data) {
        Ok(payload) => payload,
        Err(err) => return Err(err),
    };

    match create_or_retry(CreateReportInput {
        signature: payload.signature,
        title: payload.title,
        description: payload.description,
    }) {
        Ok(CreateReportResult::Created(nonce)) => Ok(HttpResponse::Created().json(nonce)),
        Ok(CreateReportResult::Retried(nonce)) => Ok(HttpResponse::Ok().json(nonce)),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::env_lock;
    use actix_web::ResponseError;
    use sqlite::Connection;
    use std::env;
    use std::sync::{Arc, Barrier};
    use std::thread;
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

    fn create_empty_db(db_path: &str) {
        let conn = sqlite::open(db_path).unwrap();
        create_schema(&conn);
    }

    fn response_status(result: Result<HttpResponse, ApiError>) -> StatusCode {
        match result {
            Ok(response) => response.status(),
            Err(err) => err.status_code(),
        }
    }

    #[test]
    fn concurrent_create_report_is_retry_safe_for_same_payload() {
        let _guard = env_lock().lock().unwrap();
        let db_path = temp_db_path("create-report-concurrent");
        create_empty_db(&db_path);
        env::set_var("DB_PATH", &db_path);

        let barrier = Arc::new(Barrier::new(2));
        let mut handles = Vec::new();

        for _ in 0..2 {
            let barrier = barrier.clone();
            handles.push(thread::spawn(move || {
                barrier.wait();

                response_status(
                    create_or_retry(CreateReportInput {
                        signature: "sig-concurrent".to_string(),
                        title: "Concurrent title".to_string(),
                        description: "Concurrent description body".to_string(),
                    })
                    .map(|result| match result {
                        CreateReportResult::Created(nonce) => HttpResponse::Created().json(nonce),
                        CreateReportResult::Retried(nonce) => HttpResponse::Ok().json(nonce),
                    })
                    .map_err(ApiError::from),
                )
            }));
        }

        let mut statuses = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();
        statuses.sort();

        assert_eq!(statuses, vec![StatusCode::OK, StatusCode::CREATED]);

        let conn = sqlite::open(&db_path).unwrap();
        conn.iterate(
            "SELECT COUNT(*) AS reports FROM reports WHERE signature = 'sig-concurrent';",
            |pairs| {
                assert_eq!(pairs[0].1.unwrap(), "1");
                true
            },
        )
        .unwrap();

        conn.iterate(
            "SELECT COUNT(*) AS nonces, MAX(nonce) AS nonce
             FROM nonces WHERE signature = 'sig-concurrent';",
            |pairs| {
                assert_eq!(pairs[0].1.unwrap(), "1");
                assert_eq!(pairs[1].1.unwrap(), "1");
                true
            },
        )
        .unwrap();
    }
}
