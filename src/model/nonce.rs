use core::result::Result;
use std::fmt;

use crate::model::db::open_connection;
use serde::Serialize;
use sqlite::{Connection, Error as sqERR, State as StateSQLite};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Nonce {
    pub uuid: String,
    pub signature: String,
    pub nonce: i32,
}

#[derive(Debug)]
pub enum NonceErr {
    DbErr(sqERR),
    NotFound(String),
}

impl fmt::Display for NonceErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NonceErr::DbErr(err) => write!(f, "Database error: {err}"),
            NonceErr::NotFound(message) => write!(f, "{message}"),
        }
    }
}

impl From<sqERR> for NonceErr {
    fn from(s: sqERR) -> Self {
        NonceErr::DbErr(s)
    }
}

impl From<String> for NonceErr {
    fn from(s: String) -> Self {
        NonceErr::NotFound(s)
    }
}

impl Nonce {
    pub fn create_in_connection(conn: &Connection, signature: String) -> Result<Nonce, NonceErr> {
        let mut db =
            conn.prepare("INSERT INTO nonces (uuid, signature, nonce) VALUES (?, ?, ?);")?;

        let nonce = Nonce::new(signature);

        db.bind((1, nonce.uuid.as_str()))?;
        db.bind((2, nonce.signature.as_str()))?;
        db.bind((3, nonce.nonce.to_string().as_str()))?;
        db.next()?;

        Ok(nonce)
    }

    pub fn increment_in_connection(&self, conn: &Connection) -> Result<Nonce, NonceErr> {
        let query = "UPDATE nonces SET nonce = :nonce WHERE uuid = :uuid";
        let mut db = conn.prepare(query)?;

        let incremented_value = self.nonce + 1;

        db.bind((":nonce", incremented_value.to_string().as_str()))?;
        db.bind((":uuid", self.uuid.as_str()))?;

        db.next()?;

        let nonce = Nonce::find_in_connection(conn, &self.signature)?;

        Ok(nonce)
    }

    pub fn find(signature: String) -> Result<Nonce, NonceErr> {
        let connection = open_connection().map_err(NonceErr::DbErr)?;
        Self::find_in_connection(&connection, &signature)
    }

    pub fn find_in_connection(conn: &Connection, signature: &str) -> Result<Nonce, NonceErr> {
        let query = "SELECT * FROM nonces where signature = :signature";
        let mut statement = conn.prepare(query)?;

        statement.bind((":signature", signature))?;

        match statement.next() {
            Ok(state) => match state {
                StateSQLite::Row => Ok(Nonce {
                    uuid: statement.read::<String, _>(0).unwrap(),
                    signature: statement.read::<String, _>(1).unwrap(),
                    nonce: statement.read::<i64, _>(2).unwrap() as i32,
                }),
                StateSQLite::Done => Err(NonceErr::NotFound("Nonce not found!".to_string())),
            },
            Err(err) => Err(NonceErr::DbErr(err)),
        }
    }

    pub fn new(signature: String) -> Nonce {
        Nonce {
            uuid: Uuid::new_v4().to_string(),
            signature,
            nonce: 1,
        }
    }
}
