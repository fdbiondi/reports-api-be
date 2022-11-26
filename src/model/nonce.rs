use core::result::Result;
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
    Empty(String),
}

impl From<sqERR> for NonceErr {
    fn from(s: sqERR) -> Self {
        NonceErr::DbErr(s)
    }
}

impl From<String> for NonceErr {
    fn from(s: String) -> Self {
        NonceErr::Empty(s)
    }
}

fn open_connection() -> Result<Connection, NonceErr> {
    let conn = sqlite::open("/usr/src/myapp/data/data.db");

    if conn.is_err() {
        let err = conn.err().unwrap();

        return Err(NonceErr::DbErr(err));
    }

    Ok(conn.unwrap())
}

// signature NVARCHAR(132) PRIMARY KEY NOT NULL
// nonce INTEGER NOT NULL
impl Nonce {
    pub fn create(signature: String) -> Result<Nonce, NonceErr> {
        let conn = open_connection()?;
        let mut db =
            conn.prepare("INSERT INTO nonces (uuid, signature, nonce) VALUES (?, ?, ?);")?;

        let nonce = Nonce::new(signature);

        db.bind((1, nonce.uuid.as_str()))?;
        db.bind((2, nonce.signature.as_str()))?;
        db.bind((3, nonce.nonce.to_string().as_str()))?;
        db.next()?;

        Ok(nonce)
    }

    pub fn increment(&self) -> Result<Nonce, NonceErr> {
        // pub fn increment(&self) -> Result<&Self, NonceErr> {

        let mut instance = match Nonce::find(self.signature.to_string()) {
            Ok(nonce) => nonce,
            Err(err) => return Err(NonceErr::Empty(err)),
        };

        instance.nonce += 1;

        let conn = open_connection()?;
        let query = "UPDATE nonces SET nonce = :nonce WHERE uuid = :uuid";
        let mut db = conn.prepare(query)?;

        db.bind((":nonce", self.nonce.to_string().as_str()))?;
        db.bind((":uuid", self.uuid.as_str()))?;

        db.next()?;

        Ok(instance)
    }

    // get nonce -> search by signature
    pub fn find(nonce: String) -> Result<Nonce, String> {
        let connection = match open_connection() {
            Ok(db) => db,
            Err(_) => return Err("Connection failed".to_string()),
        };

        let query = "SELECT * FROM nonces where signature = :signature";
        let mut statement = match connection.prepare(query) {
            Ok(stmt) => stmt,
            Err(_) => return Err("Prepare Statement failed".to_string()),
        };

        statement.bind((":signature", nonce.as_str()));

        match statement.next() {
            Ok(state) => match state {
                StateSQLite::Row => Ok(Nonce {
                    uuid: statement.read::<String, _>(0).unwrap(),
                    signature: statement.read::<String, _>(1).unwrap(),
                    nonce: statement.read::<i64, _>(1).unwrap() as i32,
                }),
                StateSQLite::Done => Err("Not Found".to_string()),
            },
            Err(_) => Err("Not Found".to_string()),
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
