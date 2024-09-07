use actix_web::{post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};

use crate::auth;
use crate::db;

#[derive(Deserialize)]
pub struct RegisterUser {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    message: String,
    token: String,
}

#[post("/register")]
pub async fn register(
    pool: web::Data<Pool>,
    user: web::Json<RegisterUser>,
) -> impl Responder {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Hash the password
    let hashed_password = match auth::hash_password(&user.password) {
        Ok(hash) => hash,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // Insert the user into the database
    if let Err(_) = db::insert_user(&client, &user.username, &hashed_password).await {
        return HttpResponse::InternalServerError().finish();
    }

    // Generate a JWT token
    let token = match auth::generate_token(&user.username) {
        Ok(token) => token,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let response = RegisterResponse {
        message: "User registered successfully".to_string(),
        token,
    };

    HttpResponse::Ok().json(response)
}