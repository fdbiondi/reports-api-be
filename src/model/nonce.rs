use core::result::Result;
use serde::Serialize;
use sqlite::{Connection, Error as sqERR};
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
}

impl From<sqERR> for NonceErr {
    fn from(s: sqERR) -> Self {
        NonceErr::DbErr(s)
    }
}

fn open_connection() -> Result<Connection, sqERR> {
    let conn = sqlite::open("../../data/mydb.sqlite")?;

    Ok(conn)
}

// signature NVARCHAR(132) PRIMARY KEY NOT NULL
// nonce INTEGER NOT NULL
impl Nonce {
    // get nonce -> search by signature

    pub fn create(signature: String) -> Result<String, NonceErr> {
        let conn = open_connection()?;
        let mut db =
            conn.prepare("INSERT INTO nonces (uuid, signature, nonce) VALUES (?, ?, ?);")?;

        let nonce = Nonce::new(signature);

        db.bind((1, nonce.uuid.as_str()))?;
        db.bind((2, nonce.signature.as_str()))?;
        db.bind((3, nonce.nonce.to_string().as_str()))?;
        db.next()?;

        Ok(nonce.uuid)
    }

    pub fn find() -> Nonce {
        Nonce::new(String::from("test"))
    }

    pub fn new(signature: String) -> Nonce {
        Nonce {
            uuid: Uuid::new_v4().to_string(),
            signature,
            nonce: 1,
        }
    }
}
