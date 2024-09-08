use actix_web::{post, web, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use log::{error, info, warn};
use crate::auth;
use crate::db;
use crate::error::AppError;

#[derive(Deserialize)]
pub struct RegisterUser {
    username: String,
    password: String,
    email: String,
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
) -> Result<HttpResponse, AppError> {
    info!("Register function called with username: {}", user.username);

    let client = pool.get().await.map_err(|e| {
        error!("Failed to get database connection: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    // Check if user already exists
    if db::user_exists(&client, &user.email, &user.username).await? {
        return Err(AppError::BadRequest("User with this email or username already exists".to_string()));
    }

    // Hash the password
    let hashed_password = auth::hash_password(&user.password).map_err(|e| {
        error!("Failed to hash password for user {}: {}", user.username, e);
        AppError::InternalServerError
    })?;

    // Insert the user into the database
    db::insert_user(&client, &user.email, &user.username, &hashed_password).await.map_err(|e| {
        error!("Failed to insert user {} into database: {}", user.username, e);
        AppError::DatabaseError(e.to_string())
    })?;

    // Generate a JWT token
    let token = auth::generate_token(&user.username).map_err(|e| {
        error!("Failed to generate JWT token for user {}: {}", user.username, e);
        AppError::InternalServerError
    })?;

    let response = RegisterResponse {
        message: "User registered successfully".to_string(),
        token,
    };

    info!("User {} registered successfully", user.username);
    Ok(HttpResponse::Ok().json(response))
}