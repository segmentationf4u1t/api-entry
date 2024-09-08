use actix_web::{post, get, web, HttpResponse};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use log::{error, info};
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

#[derive(Serialize)]
pub struct UserResponse {
    id: i64,
    username: String,
    email: String,
    created_at: chrono::DateTime<chrono::Utc>,
    avatar: Option<String>,
    status: String,
    last_login: Option<chrono::DateTime<chrono::Utc>>,
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

#[get("/user/{user_id}")]
pub async fn get_user(
    pool: web::Data<Pool>,
    user_id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    let user_id = user_id.into_inner();
    info!("Get user function called for user_id: {}", user_id);

    let client = pool.get().await.map_err(|e| {
        error!("Failed to get database connection: {}", e);
        AppError::DatabaseError(e.to_string())
    })?;

    let user = db::get_user_by_id(&client, user_id).await.map_err(|e| {
        error!("Failed to get user with id {}: {}", user_id, e);
        AppError::NotFound
    })?;

    let response = UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        created_at: user.created_at,
        avatar: user.avatar,
        status: user.status,
        last_login: user.last_login,
    };

    info!("User {} retrieved successfully", user_id);
    Ok(HttpResponse::Ok().json(response))
}