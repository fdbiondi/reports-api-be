use std::env;

use sqlite::{Connection, Error as SqlError};

pub fn open_connection() -> Result<Connection, SqlError> {
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "data/data.db".to_string());
    let connection = sqlite::open(db_path)?;

    // Let concurrent requests wait briefly on SQLite write locks instead of
    // failing immediately with "database is locked".
    connection.execute("PRAGMA busy_timeout = 1000;")?;

    Ok(connection)
}

pub fn begin_immediate_transaction(conn: &Connection) -> Result<(), SqlError> {
    conn.execute("BEGIN IMMEDIATE TRANSACTION;")
}

pub fn commit_transaction(conn: &Connection) -> Result<(), SqlError> {
    conn.execute("COMMIT;")
}

pub fn rollback_transaction(conn: &Connection) {
    let _ = conn.execute("ROLLBACK;");
}
