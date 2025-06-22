// Module declarations - these will be shared between the library and main.rs
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod logger;
pub mod middleware;
pub mod routes;
pub mod statistics;
pub mod test_utils; // Expose test utilities

// Re-export key components for easier use, e.g., by main.rs or integration tests
pub use config::AppConfig;
pub use db::establish_connection;
pub use deadpool_postgres::Pool; // Correctly re-export Pool from its source crate
pub use statistics::Statistics;
pub use middleware::rate_limiter::RateLimiter;

use actix_web::web; // Removed App, HttpServer
use std::sync::Arc;
use tokio::time::{interval, Duration};
use log::error as log_error; // aliased to avoid conflict if error module is used directly

// This function will contain the core app configuration logic
// It will be called by main.rs and by integration tests.
pub fn configure_app_routes(cfg: &mut web::ServiceConfig) {
    // This is where the .configure(routes::config) and .configure(routes::statistics::config)
    // calls from main.rs will go.
    // Example:
    cfg.configure(routes::config)
       .configure(routes::statistics::config);
}

// Placeholder for a function that might start the server, to be called from main.rs
// This is a more complex refactor, let's focus on configure_app_routes and test_utils first.
/*
pub async fn run_server() -> std::io::Result<()> {
    // ... logic from main() to load config, set up pool, logger, statistics, server ...
    // This will be a larger chunk to move.
    Ok(())
}
*/

// Function to initialize the database schema (moved from main.rs)
pub async fn init_database_schema(pool: &Pool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    db::create_statistics_tables(&client).await?;
    Ok(())
}

// Function to start background tasks (moved and adapted from main.rs)
pub fn start_background_tasks(
    statistics: Arc<Statistics>,
    pool: Pool,
    app_start_time: chrono::DateTime<chrono::Utc>,
) {
    // Task to periodically save statistics
    let stats_for_save = Arc::clone(&statistics);
    let pool_for_save = pool.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60 * 5)); // Save every 5 minutes (as in main)
        loop {
            interval.tick().await;
            if let Err(e) = stats_for_save.save(&pool_for_save).await {
                log_error!("Failed to save statistics: {}", e);
            }
        }
    });

    // Task to update uptime
    let stats_for_uptime = Arc::clone(&statistics);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await; // Update every minute
            let uptime = (chrono::Utc::now() - app_start_time).num_seconds() as f64;
            stats_for_uptime.update_uptime(uptime).await; // Added .await as update_uptime is async
        }
    });
}
