use actix_web::{get, HttpResponse, Responder};

#[get("/rate-test")]
pub async fn rate_test() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "message": "Rate test endpoint" }))
}