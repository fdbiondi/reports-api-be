use std::env;

use sqlite::{Connection, Error as SqlError};

pub fn open_connection() -> Result<Connection, SqlError> {
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "data/data.db".to_string());
    sqlite::open(db_path)
}
