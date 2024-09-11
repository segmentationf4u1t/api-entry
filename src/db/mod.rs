use deadpool_postgres::{Config, Pool, Runtime, Client};
use tokio_postgres::NoTls;
use crate::{config::DatabaseConfig, statistics::StatisticsData};
use crate::auth::User;
use crate::error::AppError;
use chrono::{DateTime, Utc, NaiveDateTime};
use validator::validate_email;
use serde_json::Value;
use tokio_postgres::types::Json;
use std::collections::HashMap;

pub async fn user_exists(client: &Client, email: &str, username: &str) -> Result<bool, AppError> {
    let row = client
        .query_one(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 OR username = $2)",
            &[&email, &username],
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(row.get(0))
}

pub async fn insert_user(client: &Client, email: &str, username: &str, hashed_password: &str) -> Result<User, AppError> {
    // Validate email
    if !validate_email(email) {
        return Err(AppError::BadRequest("Invalid email format".to_string()));
    }

    // Validate username (example: alphanumeric, 3-20 characters)
    if !username.chars().all(|c| c.is_alphanumeric()) || username.len() < 3 || username.len() > 20 {
        return Err(AppError::BadRequest("Invalid username format".to_string()));
    }

    // Validate hashed password length (assuming bcrypt, which is typically 60 characters)
    if hashed_password.len() != 60 {
        return Err(AppError::BadRequest("Invalid password hash".to_string()));
    }

    // Check if user already exists
    if user_exists(client, email, username).await? {
        return Err(AppError::BadRequest("User with this email or username already exists".to_string()));
    }

    // Proceed with insertion...
    let row = client.query_one(
        "INSERT INTO users (email, username, password) VALUES ($1, $2, $3) RETURNING id, email, username, created_at, avatar, tokens, status, permissions, last_login",
        &[&email, &username, &hashed_password],
    ).await.map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(User {
        id: row.get(0),
        email: row.get(1),
        username: row.get(2),
        created_at: row.get::<_, DateTime<Utc>>(3),
        avatar: row.get(4),
        tokens: row.get::<_, Option<Value>>(5),
        status: row.get(6),
        permissions: row.get::<_, Option<Value>>(7),
        last_login: row.get::<_, Option<DateTime<Utc>>>(8),
    })
}

pub fn establish_connection(config: &DatabaseConfig) -> Result<Pool, Box<dyn std::error::Error>> {
    let mut cfg = Config::new();
    cfg.dbname = Some(config.url.clone());
    cfg.user = Some(config.username.clone());
    cfg.password = Some(config.password.clone());
    cfg.host = Some(config.host.clone());
    cfg.port = Some(config.port);
    
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;
    pool.resize(config.max_connections as usize);
    Ok(pool)
}

pub async fn test_connection(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    // Get a client from the pool
    let client: Client = pool.get().await?;

    // Execute a simple query
    let result = client.query_one("SELECT 1", &[]).await?;

    // Check if the result is as expected
    let value: i32 = result.get(0);
    if value == 1 {
        println!("Database connection test successful!");
        Ok(())
    } else {
        Err("Unexpected result from database test query".into())
    }
}

pub async fn get_user_by_id(client: &Client, user_id: i64) -> Result<User, AppError> {
    let row = client
        .query_one(
            "SELECT id, username, email, created_at, avatar, status, last_login FROM users WHERE id = $1",
            &[&user_id],
        )
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        created_at: row.get("created_at"),
        tokens: None,
        permissions: None,
        avatar: row.get("avatar"),
        status: row.get("status"),
        last_login: row.get("last_login"),
    })
}

pub async fn update_last_login(client: &Client, user_id: i64) -> Result<(), tokio_postgres::Error> {
    client.execute(
        "UPDATE users SET last_login = NOW() WHERE id = $1",
        &[&user_id],
    ).await?;
    Ok(())
}

pub async fn create_statistics_tables(client: &Client) -> Result<(), tokio_postgres::Error> {
    client
        .batch_execute(
            "
            CREATE TABLE IF NOT EXISTS api_statistics (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                total_requests BIGINT NOT NULL,
                avg_response_time DOUBLE PRECISION NOT NULL,
                error_rate DOUBLE PRECISION NOT NULL,
                uptime DOUBLE PRECISION NOT NULL,
                register_requests BIGINT NOT NULL,
                register_success BIGINT NOT NULL,
                get_user_requests BIGINT NOT NULL,
                get_user_success BIGINT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS api_traffic_distribution (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                route TEXT NOT NULL,
                count BIGINT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS api_request_log (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                method TEXT NOT NULL,
                endpoint TEXT NOT NULL,
                status SMALLINT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS api_error_log (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                message TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_api_statistics_timestamp ON api_statistics(timestamp);
            CREATE INDEX IF NOT EXISTS idx_api_traffic_distribution_timestamp ON api_traffic_distribution(timestamp);
            CREATE INDEX IF NOT EXISTS idx_api_request_log_timestamp ON api_request_log(timestamp);
            CREATE INDEX IF NOT EXISTS idx_api_error_log_timestamp ON api_error_log(timestamp);
            "
        )
        .await?;
    Ok(())
}

pub async fn insert_statistics(client: &Client, data: &StatisticsData) -> Result<(), tokio_postgres::Error> {
    let timestamp = Utc::now();

    // Insert main statistics
    client
        .execute(
            "INSERT INTO api_statistics (timestamp, total_requests, avg_response_time, error_rate, uptime, register_requests, register_success, get_user_requests, get_user_success)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            &[&timestamp, &data.total_requests, &data.avg_response_time, &data.error_rate, &data.uptime, 
              &(data.register_requests as i64), &(data.register_success as i64), 
              &(data.get_user_requests as i64), &(data.get_user_success as i64)],
        )
        .await?;

    // Insert traffic distribution
    for (route, count) in &data.traffic_distribution {
        client
            .execute(
                "INSERT INTO api_traffic_distribution (timestamp, route, count)
                 VALUES ($1, $2, $3)",
                &[&timestamp, route, &(*count as i64)],
            )
            .await?;
    }

    // Insert last requests
    for request in &data.last_requests {
        client
            .execute(
                "INSERT INTO api_request_log (timestamp, method, endpoint, status)
                 VALUES ($1, $2, $3, $4)",
                &[&request.timestamp, &request.method, &request.endpoint, &(request.status as i16)],
            )
            .await?;
    }

    // Insert error log
    for error in &data.error_log {
        client
            .execute(
                "INSERT INTO api_error_log (timestamp, message)
                 VALUES ($1, $2)",
                &[&error.timestamp, &error.message],
            )
            .await?;
    }

    Ok(())
}




