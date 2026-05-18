use std::{fmt, str::FromStr};

use actix_web::{http::StatusCode, HttpResponse, ResponseError};

use crate::model::db::open_connection;
use serde::Serialize;
use sqlite::{Connection, Error as sqERR, State as StateSQLite};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Serialize, EnumString, Display, Eq, PartialEq, Debug)]
pub enum ReportState {
    InProgress,
    Completed,
    Failed,
}

#[derive(Serialize)]
pub struct Report {
    pub uuid: String,
    pub signature: String,
    pub description: String,
    pub title: String,
    pub state: ReportState,
}

#[derive(Debug)]
pub enum ReportErr {
    DbErr(sqERR),
    NotFound(String),
}

impl fmt::Display for ReportErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReportErr::DbErr(err) => write!(f, "Database error: {err}"),
            ReportErr::NotFound(message) => write!(f, "{message}"),
        }
    }
}

impl From<sqERR> for ReportErr {
    fn from(s: sqERR) -> Self {
        ReportErr::DbErr(s)
    }
}

impl From<String> for ReportErr {
    fn from(s: String) -> Self {
        ReportErr::NotFound(s)
    }
}

impl ResponseError for ReportErr {
    fn error_response(&self) -> HttpResponse {
        #[derive(Serialize)]
        struct ErrorResponse {
            code: String,
            error: String,
        }

        HttpResponse::build(self.status_code()).json(ErrorResponse {
            code: match self {
                ReportErr::NotFound(_) => "NOT_FOUND".to_string(),
                ReportErr::DbErr(_) => "INTERNAL_ERROR".to_string(),
            },
            error: self.to_string(),
        })
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ReportErr::NotFound(_) => StatusCode::NOT_FOUND,
            ReportErr::DbErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// signature NVARCHAR(132) PRIMARY KEY NOT NULL
// description TEXT NOT NULL
// title NVARCHAR(50) NOT NULL)
impl Report {
    pub fn find(signature: String) -> Result<Report, ReportErr> {
        let connection = open_connection().map_err(ReportErr::DbErr)?;
        Self::find_in_connection(&connection, &signature)
    }

    pub fn create_in_connection(
        conn: &Connection,
        signature: String,
        title: String,
        description: String,
    ) -> Result<Report, ReportErr> {
        let query = "INSERT INTO reports (uuid, signature, description, title, state) VALUES (:uuid, :signature, :description, :title, :state);";
        let mut db = conn.prepare(query)?;

        let report = Report::new(signature, title, description);

        db.bind((":uuid", report.uuid.as_str()))?;
        db.bind((":signature", report.signature.as_str()))?;
        db.bind((":description", report.description.as_str()))?;
        db.bind((":title", report.title.as_str()))?;
        db.bind((":state", report.state.to_string().as_str()))?;
        db.next()?;

        Ok(report)
    }

    pub fn find_in_connection(conn: &Connection, signature: &str) -> Result<Report, ReportErr> {
        let query = "SELECT * FROM reports WHERE signature = :signature";
        let mut statement = conn.prepare(query)?;

        statement.bind((":signature", signature))?;

        match statement.next() {
            Ok(state) => match state {
                StateSQLite::Row => Ok(Report {
                    uuid: statement.read::<String, _>(0).unwrap(),
                    signature: statement.read::<String, _>(1).unwrap().to_string(),
                    description: statement.read::<String, _>(2).unwrap(),
                    title: statement.read::<String, _>(3).unwrap(),
                    state: ReportState::from_str(&statement.read::<String, _>(4).unwrap()).unwrap(),
                }),
                StateSQLite::Done => Err(ReportErr::NotFound(String::from("Report Not found!"))),
            },
            Err(err) => Err(ReportErr::DbErr(err)),
        }
    }

    pub fn matches_payload(&self, title: &str, description: &str) -> bool {
        self.title == title && self.description == description
    }

    pub fn new(signature: String, title: String, description: String) -> Report {
        Report {
            uuid: Uuid::new_v4().to_string(),
            signature,
            description,
            title,
            state: ReportState::InProgress,
        }
    }
}
