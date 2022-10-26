use actix_web::{
	get
};

#[get("/nonces/{signature}")]
pub async fn get_report() -> String {
    // TODO
    String::from("not implemented yet")
}
