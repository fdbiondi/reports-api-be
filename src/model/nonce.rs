use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Report {
	pub uuid: String,
	pub signature: String,
	pub nonce: i32
}

// signature NVARCHAR(132) PRIMARY KEY NOT NULL
// nonce INTEGER NOT NULL
impl Nonce {
	pub fn new(signature) -> Report {
		Nonce {
			uuid: Uuid::new_v4().to_string(),
			signature,
			nonce: 1
		}
	}
}
