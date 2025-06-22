use serde::Deserialize;
use config::{Config, ConfigError, File, Environment};

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
    pub file: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub rate_limit: RateLimitConfig,
    pub log: LogConfig,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            .add_source(File::with_name("config.toml").required(false)) // Allow config file to be optional
            .add_source(File::with_name("example.config.toml")); // Fallback to example config

        // For test environments, allow overriding database settings with environment variables
        // We check for a common test indicator, or you can set a specific env var to signal a test run.
        if std::env::var("RUN_ENV").unwrap_or_default() == "test" {
            builder = builder.add_source(
                Environment::with_prefix("TEST_APP") // Example: TEST_APP_DATABASE_URL
                    .separator("_")
                    .try_parsing(true)
            );
        } else {
            // For non-test environments, you might want to load other sources or enforce stricter rules
            // For example, require a specific config file or use a different set of environment variables.
            builder = builder.add_source(
                Environment::with_prefix("APP") // Example: APP_DATABASE_URL
                    .separator("_")
                    .try_parsing(true)
            );
        }

        let config = builder.build()?;
        config.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_load_config_defaults() {
        // Test loading from example.config.toml
        let config = AppConfig::new();
        assert!(config.is_ok());
        let app_config = config.unwrap();
        assert_eq!(app_config.server.port, 8080); // Assuming example.config.toml has this
    }

    #[test]
    fn test_load_config_with_env_override() {
        env::set_var("RUN_ENV", "test");
        env::set_var("TEST_APP_DATABASE_URL", "postgres://test_user:test_pass@localhost:5433/test_db");
        env::set_var("TEST_APP_SERVER_PORT", "9090");

        let config_result = AppConfig::new();
        assert!(config_result.is_ok());
        let app_config = config_result.unwrap();

        assert_eq!(app_config.database.url, "postgres://test_user:test_pass@localhost:5433/test_db");
        assert_eq!(app_config.server.port, 9090);

        env::remove_var("RUN_ENV");
        env::remove_var("TEST_APP_DATABASE_URL");
        env::remove_var("TEST_APP_SERVER_PORT");
    }

    #[test]
    fn test_config_still_loads_if_config_toml_missing() {
        // Temporarily rename config.toml if it exists to simulate it being missing
        let config_existed = std::fs::rename("config.toml", "config.toml.backup").is_ok();

        let config = AppConfig::new();
        assert!(config.is_ok());

        // Restore config.toml if it was renamed
        if config_existed {
            std::fs::rename("config.toml.backup", "config.toml").unwrap();
        }
    }
}