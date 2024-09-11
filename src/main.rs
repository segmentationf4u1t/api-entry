mod auth;
mod config;
mod db;
mod error;
mod logger;
mod middleware;
mod routes;
mod statistics;
//use actix_web::dev::Service;
use actix_web::{App, HttpServer};
use config::AppConfig;
use db::establish_connection;

use middleware::rate_limiter::RateLimiter;
use log4rs;
use actix_web::web;
use actix_web::dev::Service; // Change this line
use deadpool_postgres::Pool;
use statistics::Statistics;
use std::sync::Arc;
use tokio;
use log::error;
use tokio::time::{interval, Duration};
use futures::FutureExt;

use chrono::Utc;
use actix_cors::Cors;
use actix_web::dev::ServiceResponse;
use actix_web::dev::ServiceRequest;

async fn init_database(pool: &Pool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    db::create_statistics_tables(&client).await?;
    Ok(())
}

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
    let statistics = Arc::new(Statistics::new());

    // Create statistics table if it doesn't exist
    if let Ok(client) = pool.get().await {
        if let Err(e) = db::create_statistics_tables(&client).await {
            error!("Failed to create statistics table: {}", e);
        }
    }

    // Start a background task to periodically save statistics
    let stats_clone = Arc::clone(&statistics);
    let pool_for_stats = pool.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(300)); // Save every 5 minutes
        loop {
            interval.tick().await;
            if let Err(e) = stats_clone.save(&pool_for_stats).await {
                error!("Failed to save statistics: {}", e);
            }
        }
    });

    // Start a background task to update uptime
    let stats_clone = Arc::clone(&statistics);
    let start_time = Utc::now();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await; // Update every minute
            let uptime = (Utc::now() - start_time).num_seconds() as f64;
            stats_clone.update_uptime(uptime);
        }
    });

    // Initialize database
    init_database(&pool).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    // Create and run the HTTP server
    let pool_for_server = pool.clone();
    HttpServer::new(move || {
        let stats = web::Data::new(Arc::clone(&statistics));
        let pool = pool_for_server.clone();
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(Cors::permissive())
            .wrap(rate_limiter.clone())
            .wrap_fn(|req, srv| {
                let stats = req.app_data::<web::Data<Arc<Statistics>>>().unwrap().clone();
                let start_time = Utc::now();
                let req_method = req.method().clone();
                let req_path = req.path().to_owned();
                srv.call(req).map(move |res| {
                    if let Ok(res) = &res {
                        let end_time = Utc::now();
                        let duration = (end_time - start_time).num_milliseconds() as f64;
                        stats.get_ref().log_request(
                            req_method.as_str(),
                            &req_path,
                            res.status().as_u16(),
                            duration,
                        );
                    }
                    res
                })
            })
            .app_data(pool.clone())
            .app_data(stats.clone())
            .configure(routes::config)
            .configure(routes::statistics::config)
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}