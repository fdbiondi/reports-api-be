use actix_web::{
	get,
	post
};

#[get("/reports/{signature}")]
pub async fn get_report() -> String {
    // TODO
    String::from("not implemented yet")
}

#[post("/reports")]
pub async fn create_report() -> String {
    // TODO
    String::from("not implemented yet")
}
