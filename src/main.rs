mod auth;
mod config;
mod db;
mod error;
mod logger;
mod middleware;
mod routes;
mod statistics;
use actix_web::dev::Service;
use actix_web::{App, HttpServer};
use config::AppConfig;
use db::establish_connection;
use middleware::rate_limiter::RateLimiter;
use log4rs;
use actix_web::web;
use statistics::Statistics;
use std::sync::Arc;
use tokio;
use log::error;
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

    let pool_clone = pool.clone();
    let statistics = Arc::new(Statistics::new(pool.clone()));

    // Create statistics table if it doesn't exist
    if let Ok(client) = pool.get().await {
        if let Err(e) = db::create_statistics_table(&client).await {
            error!("Failed to create statistics table: {}", e);
        }
    }

    // Start a background task to periodically save statistics
    let stats_clone = Arc::clone(&statistics);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            stats_clone.save().await;
        }
    });

    // Create and run the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool_clone.clone()))
            .app_data(web::Data::new(Arc::clone(&statistics)))
            .wrap(rate_limiter.clone()) // Apply rate limiter middleware
            .wrap_fn(|req, srv| {
                let stats = req.app_data::<web::Data<Arc<Statistics>>>().unwrap().clone();
                let path = req.path().to_string();
                let fut = srv.call(req);
                async move {
                    let res = fut.await?;
                    stats.increment(&format!("request_{}", path)).await;
                    Ok(res)
                }
            })
            .configure(routes::config) // Configure routes
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}