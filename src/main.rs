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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration
    let config = AppConfig::new().expect("Failed to load configuration");

    
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize logger");
    // Establish database connection
    let pool = establish_connection(&config.database)
        .expect("Failed to create pool");

    // Create rate limiter
    let rate_limiter = RateLimiter::new(
        config.rate_limit.requests_per_second,
        config.rate_limit.burst_size,
    );

    log::info!("Starting server at {}:{}", config.server.host, config.server.port);

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .wrap(rate_limiter.clone())
            .configure(routes::config)
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}