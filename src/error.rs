use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

use crate::model::nonce::NonceErr;
use crate::model::report::ReportErr;

#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorDetail {
    field: String,
    issue: String,
}

impl ApiErrorDetail {
    pub fn new(field: impl Into<String>, issue: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            issue: issue.into(),
        }
    }
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
    details: Option<Vec<ApiErrorDetail>>,
}

#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Vec<ApiErrorDetail>>,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn invalid_json(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "INVALID_JSON", message)
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "NOT_FOUND", message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, "CONFLICT", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message)
    }

    pub fn with_details(mut self, details: Vec<ApiErrorDetail>) -> Self {
        // Keep base error context and let handlers append request-specific detail.
        match self.details.as_mut() {
            Some(existing) => existing.extend(details),
            None => self.details = Some(details),
        }
        self
    }

    pub fn db_failure(operation: impl Into<String>, resource: impl Into<String>) -> Self {
        Self::internal("Database operation failed").with_details(vec![
            ApiErrorDetail::new("operation", operation),
            ApiErrorDetail::new("resource", resource),
        ])
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status).json(ErrorResponse {
            code: self.code.to_string(),
            error: self.message.clone(),
            details: self.details.clone(),
        })
    }
}

impl From<ReportErr> for ApiError {
    fn from(value: ReportErr) -> Self {
        match value {
            ReportErr::NotFound(message) => ApiError::not_found(message)
                .with_details(vec![ApiErrorDetail::new("resource", "report")]),
            ReportErr::DbErr(_) => ApiError::db_failure("fetch", "report"),
        }
    }
}

impl From<NonceErr> for ApiError {
    fn from(value: NonceErr) -> Self {
        match value {
            NonceErr::NotFound(message) => ApiError::not_found(message)
                .with_details(vec![ApiErrorDetail::new("resource", "nonce")]),
            NonceErr::DbErr(_) => ApiError::db_failure("fetch", "nonce"),
        }
    }
}
