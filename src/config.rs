use serde::Deserialize;
use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub file: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub rate_limit: RateLimitConfig,
    pub log: LogConfig,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config.toml"))
            .build()?;

        config.try_deserialize()
    }
}