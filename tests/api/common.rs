use my_actix_api::config::AppConfig;
use my_actix_api::main_routes; // Assuming your main router setup is in a function called `main_routes` in lib.rs or main.rs
use my_actix_api::logger; // Assuming logger setup is accessible
use my_actix_api::db; // For Pool
use my_actix_api::middleware::rate_limiter::RateLimiter; // Import RateLimiter

use actix_web::{test, web, App, HttpServer};
use deadpool_postgres::Pool;
use std::sync::Once;
use std::net::TcpListener;

// Ensure this path is correct based on where setup_test_db is.
// If helpers is in src/tests/mod.rs and helpers.rs, this won't work directly
// as src/tests is not a module visible to integration tests in `tests/`.
// `setup_test_db` needs to be part of the `my_actix_api` crate's public API (e.g., in a `my_actix_api::tests_utils` module)
// or we replicate/call it carefully.
// For now, let's assume `my_actix_api::tests::helpers::setup_test_db` is made available for integration tests.
// This typically requires making `src/tests/mod.rs` and `src/tests/helpers.rs` part of the library build
// when the `test` feature or profile is active, or moving test utils to a `my_actix_api::test_utils` module.

// A simpler way for now if `setup_test_db` cannot be directly called:
// The `setup_test_db` is in `my_actix_api::src::tests::helpers`.
// We need to expose it or a similar function from `my_actix_api` lib.
// Let's assume we will add a `pub mod test_utils;` to `my_actix_api::lib.rs` (or main.rs if it's a binary)
// and move the content of `src/tests/helpers.rs` there.

use my_actix_api::test_utils::setup_test_db; // Use the new test_utils module
use my_actix_api::{AppConfig, Pool, RateLimiter, Statistics, configure_app_routes}; // Use items from lib.rs
use actix_web::{test, web, App, HttpServer}; // HttpServer is needed to run the server
use std::sync::{Arc, Once};
use std::net::TcpListener;
use chrono::Utc; // For app_start_time for background tasks
use my_actix_api::start_background_tasks; // Import the background task starter

static INIT: Once = Once::new();

fn initialize_test_logging() {
    INIT.call_once(|| {
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "actix_web=warn,my_actix_api=debug"); // Adjusted log levels
        }
        // Attempt to initialize env_logger, but don't panic if it fails (e.g., if already initialized)
        env_logger::try_init().ok();
        println!("Test logging initialized (env_logger).");
    });
}

pub struct TestApp {
    pub address: String,
    pub db_pool: Pool,
    // pub http_client: reqwest::Client, // Can be added if needed for all tests
}

/// Spawns the application on a random available port and sets up a test database.
pub async fn spawn_app() -> TestApp {
    initialize_test_logging();
    std::env::set_var("RUN_ENV", "test");

    // Setup test database using the utility from the library crate
    let db_pool = setup_test_db().await.expect("Failed to set up test DB. Ensure DB server is running and accessible.");

    // Load configuration
    let app_config = AppConfig::new().expect("Failed to load test configuration.");

    // Create Statistics manager instance
    let statistics_manager = Arc::new(Statistics::new());

    // It's important that the test database schema is also initialized
    // if `setup_test_db` only runs migrations but doesn't handle `create_statistics_tables`
    // However, `setup_test_db` *should* run all migrations, which create all tables.
    // The `init_database_schema` in lib.rs also calls `create_statistics_tables`,
    // which might be redundant if migrations already do this.
    // For tests, `setup_test_db` running the main migration SQL should be sufficient.

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // Clone Arcs and other data needed for the server factory
    let server_db_pool = db_pool.clone();
    let server_statistics = Arc::clone(&statistics_manager);
    let server_app_config = app_config.clone();

    // Start background tasks for the test server instance
    // Note: For isolated tests, you might not want these running, or want control over them.
    // For now, we mirror main.rs behavior.
    let app_start_time = Utc::now();
    start_background_tasks(Arc::clone(&statistics_manager), db_pool.clone(), app_start_time);


    let server = HttpServer::new(move || {
        let rate_limiter_config = server_app_config.rate_limit.clone();
        let rate_limiter = RateLimiter::new(rate_limiter_config.requests_per_second, rate_limiter_config.burst_size);

        App::new()
            .wrap(actix_web::middleware::Logger::default()) // Actix default logger
            .wrap(actix_cors::Cors::permissive())
            .wrap(rate_limiter)
            // Custom logging middleware for statistics - simplified for spawn_app context
            // The full one from main.rs can be used if preferred.
            .wrap_fn(move |req, srv| {
                let stats = Arc::clone(&server_statistics);
                let start = Utc::now();
                let fut = srv.call(req);
                async move {
                    let res = fut.await?;
                    let duration = (Utc::now() - start).num_milliseconds() as f64;
                    stats.log_request(
                        res.request().method().as_str().to_string(),
                        res.request().path().to_string(),
                        res.status().as_u16(),
                        duration,
                    ).await; // log_request is async
                    Ok(res)
                }
            })
            .app_data(web::Data::new(server_db_pool.clone()))
            .app_data(web::Data::new(Arc::clone(&server_statistics)))
            .app_data(web::Data::new(server_app_config.clone()))
            .configure(configure_app_routes) // Use the centralized route configurator
    })
    .listen(listener) // Listen on the TcpListener
    .expect("Failed to bind server to listener")
    .run();

    // Run the server in a separate Tokio task
    tokio::spawn(server);

    TestApp { address, db_pool }
}
