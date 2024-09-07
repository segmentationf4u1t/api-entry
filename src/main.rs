use actix_web::{web, App, HttpServer};
use deadpool_postgres::{Config, Pool, Runtime};
use log::{LevelFilter, SetLoggerError};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config as LogConfig, Root},
    encode::pattern::PatternEncoder,
};
use tokio_postgres::NoTls;
mod auth;
mod db;
mod error;
mod logger;
mod middleware;
mod routes;

fn setup_logger() -> Result<(), SetLoggerError> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}\n")))
        .build("rate_limit.log")
        .unwrap();

    let config = LogConfig::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(config)?;

    Ok(())
}

fn create_pool() -> Result<deadpool_postgres::Pool, deadpool_postgres::CreatePoolError> {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.dbname = Some("your_database_name".to_string());
    cfg.user = Some("your_username".to_string());
    cfg.password = Some("your_password".to_string());
    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_logger().expect("Failed to set up logger");

    let pool = create_pool().expect("Failed to create database pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::rate_limiter::RateLimiter::default())
            .wrap(logger::Logger)
            .service(web::scope("/api").configure(routes::config))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}