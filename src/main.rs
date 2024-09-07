mod auth;
mod config;
mod db;
mod error;
mod logger;
mod middleware;
mod routes;

use actix_web::{App, HttpServer};
use config::AppConfig;
use db::establish_connection;
use middleware::rate_limiter::RateLimiter;
use log4rs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load application configuration from config file
    let config = AppConfig::new().expect("Failed to load configuration");
    // Establish database connection pool using the configuration
   
    // Initialize the logger using log4rs and the log4rs.yaml configuration file
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize logger");

    // Establish database connection pool using the configuration
    let pool = establish_connection(&config.database)
        .expect("Failed to create pool");
    db::test_connection(&pool).await.expect("Failed to test database connection");
    // Create rate limiter middleware with configured settings
    let rate_limiter = RateLimiter::new(
        config.rate_limit.requests_per_second,
        config.rate_limit.burst_size,
    );

    // Log server start information
    log::info!("Starting server at {}:{}", config.server.host, config.server.port);

    // Create and run the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone()) // Add database pool to app data
            .wrap(rate_limiter.clone()) // Apply rate limiter middleware
            .configure(routes::config) // Configure routes
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}