// Use items from the library crate
use my_actix_api::{
    AppConfig,
    // Pool, // Pool is used via db_pool which is typed, direct import not needed
    Statistics,
    RateLimiter,
    establish_connection,
    configure_app_routes,
    init_database_schema,
    start_background_tasks,
    // db,
    // routes,
    // middleware,
    // logger, // logger module itself not directly used, log macros are via `log` crate
};

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use chrono::Utc;
use log; // Ensure log macros are available
use log4rs; // For logger initialization
use futures::FutureExt; // For .map on futures
use actix_web::dev::Service; // For srv.call

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load application configuration using AppConfig from the library
    let app_config = AppConfig::new().expect("Failed to load configuration");
   
    // Initialize the logger (assuming log4rs.yaml is still the config file)
    // If logger::init_logger is now in lib.rs, call that.
    // For now, keeping direct init here.
    log4rs::init_file("log4rs.yaml", Default::default()).expect("Failed to initialize logger");

    // Establish database connection pool using establish_connection from the library
    let db_pool = establish_connection(&app_config.database)
        .expect("Failed to create database pool");

    // Test database connection (optional, db might be a private module now)
    // my_actix_api::db::test_connection(&db_pool).await.expect("Failed to test database connection");
    // This specific test_connection might not be pub. If it's important, it should be.
    // For now, assume establish_connection is enough indication.

    // Create rate limiter middleware
    let rate_limiter = RateLimiter::new(
        app_config.rate_limit.requests_per_second,
        app_config.rate_limit.burst_size,
    );

    log::info!("Starting server at {}:{}", app_config.server.host, app_config.server.port);

    let statistics_manager = Arc::new(Statistics::new());

    // Initialize database schema (e.g., create tables if they don't exist)
    // This replaces the direct call to db::create_statistics_tables
    if let Err(e) = init_database_schema(&db_pool).await {
        log::error!("Failed to initialize database schema: {}", e);
        // Depending on severity, might want to exit or handle differently
    }

    let app_start_time = Utc::now();
    start_background_tasks(Arc::clone(&statistics_manager), db_pool.clone(), app_start_time);

    // Create and run the HTTP server
    let server_db_pool = db_pool.clone();
    let server_statistics = Arc::clone(&statistics_manager);
    let server_app_config = app_config.clone(); // Clone AppConfig for the factory

    HttpServer::new(move || {
        // Clone Arcs needed for the App factory here
        // These are the "original" Arcs/values for this factory instance
        let factory_db_pool = server_db_pool.clone();
        let factory_statistics_arc = Arc::clone(&server_statistics);
        let factory_app_config = server_app_config.clone();
        let factory_rate_limiter = rate_limiter.clone();

        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_cors::Cors::permissive())
            .wrap(factory_rate_limiter)
            .wrap_fn({ // Use a block to scope the clone for wrap_fn
                let stats_clone_for_wrap_service = Arc::clone(&factory_statistics_arc); // Clone #1 for the service factory
                move |req, srv| { // This closure is the service factory
                    let stats_clone_for_log_task = Arc::clone(&stats_clone_for_wrap_service); // Clone #2 for the spawned task
                    let req_method_owned = req.method().clone().to_string();
                    let req_path_owned = req.path().to_owned();
                    let start_time = Utc::now();

                    srv.call(req).map(move |res: Result<actix_web::dev::ServiceResponse<_>, actix_web::Error>| {
                        if let Ok(res_ok) = &res {
                            let end_time = Utc::now();
                            let duration = (end_time - start_time).num_milliseconds() as f64;
                            let status_code = res_ok.status().as_u16();
                            tokio::spawn(async move {
                                stats_clone_for_log_task.log_request(
                                    &req_method_owned,
                                    &req_path_owned,
                                    status_code,
                                    duration,
                                ).await;
                            });
                        }
                        res
                    })
                }
            }) // End of wrap_fn
            .app_data(web::Data::new(factory_db_pool))
            .app_data(web::Data::new(factory_statistics_arc)) // Use the original factory_statistics_arc
            .app_data(web::Data::new(factory_app_config))
            .configure(configure_app_routes)
    })
    .bind(format!("{}:{}", app_config.server.host, app_config.server.port))?
    .run()
    .await
}