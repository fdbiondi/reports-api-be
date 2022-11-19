use serde::Serialize;
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

// signature NVARCHAR(132) PRIMARY KEY NOT NULL
// description TEXT NOT NULL
// title NVARCHAR(50) NOT NULL)
impl Report {
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
