use serde::Serialize;
use sqlite::{Connection, Error as sqERR};
use strum_macros::{Display, EnumString};
use uuid::Uuid;

#[derive(Serialize, Display, Eq, PartialEq)]
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
}

impl From<sqERR> for ReportErr {
    fn from(s: sqERR) -> Self {
        ReportErr::DbErr(s)
    }
}

fn open_connection() -> Result<Connection, sqERR> {
    let conn = sqlite::open("../../data/mydb.sqlite")?;

    Ok(conn)
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
        let mut db =
            conn.prepare("INSERT INTO reports (uuid, signature, description, title, state) VALUES (?, ?, ?, ?, ?);")?;

        let report = Report::new(signature, title, description);

        db.bind(1, report.uuid.as_bytes())?;
        db.bind(2, report.signature.as_bytes())?;
        db.bind(3, report.description.as_bytes())?;
        db.bind(4, report.title.as_bytes())?;
        db.bind(5, report.state.to_string().as_bytes())?;
        db.next()?;

        Ok(report)
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
