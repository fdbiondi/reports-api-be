use std::str::FromStr;

use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse, ResponseError,
};

use serde::Serialize;
use sqlite::{Connection, Error as sqERR};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Serialize, EnumString, Display, Eq, PartialEq)]
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

#[derive(Debug, Display)]
pub enum ReportErr {
    DbErr(sqERR),
    Empty(String),
}

impl From<sqERR> for ReportErr {
    fn from(s: sqERR) -> Self {
        ReportErr::DbErr(s)
    }
}

impl From<String> for ReportErr {
    fn from(s: String) -> Self {
        ReportErr::Empty(s)
    }
}

impl ResponseError for ReportErr {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            _ => StatusCode::NOT_FOUND,
        }
    }
}

fn open_connection() -> Result<Connection, ReportErr> {
    let conn = sqlite::open("/usr/src/myapp/data/data.db");

    if conn.is_err() {
        let err = conn.err().unwrap();

        return Err(ReportErr::DbErr(err));
    }

    Ok(conn.unwrap())
}

// signature NVARCHAR(132) PRIMARY KEY NOT NULL
// description TEXT NOT NULL
// title NVARCHAR(50) NOT NULL)
impl Report {
    pub fn create(
        signature: String,
        title: String,
        description: String,
    ) -> Result<Report, ReportErr> {
        let conn = open_connection()?;
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

    pub fn find(signature: String) -> Result<Report, ReportErr> {
        let connection = match open_connection() {
            Ok(db) => db,
            Err(err) => return Err(err),
        };

        let query = "SELECT * FROM reports WHERE signature = :signature";
        let mut statement = connection.prepare(query)?;

        statement.bind((":signature", signature.as_str()))?;

        match statement.next() {
            Ok(_) => Ok(Report {
                uuid: statement.read::<String, _>(0).unwrap(),
                signature: statement.read::<String, _>(1).unwrap().to_string(),
                description: statement.read::<String, _>(2).unwrap(),
                title: statement.read::<String, _>(3).unwrap(),
                state: ReportState::from_str(&statement.read::<String, _>(4).unwrap()).unwrap(),
            }),
            Err(err) => return Err(ReportErr::DbErr(err)),
        }
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
