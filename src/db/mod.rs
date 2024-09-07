use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use crate::config::DatabaseConfig;

pub async fn insert_user(client: &deadpool_postgres::Client, username: &str, hashed_password: &str) -> Result<(), tokio_postgres::Error> {
    client.execute(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
        &[&username, &hashed_password],
    ).await?;
    Ok(())
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