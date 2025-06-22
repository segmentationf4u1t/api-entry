// This file contains test utilities, primarily for setting up a test database.
// It's intended to be used by integration tests.

// Note: `crate::` here refers to `my_actix_api` (the library crate).
use crate::config::AppConfig;
use crate::db; // Use db module from the library
use deadpool_postgres::{Client, Pool}; // Import Pool directly from deadpool_postgres
use std::fs;
use std::path::Path;

/// Sets up a test database by running migrations.
///
/// Returns a connection pool to the test database.
pub async fn setup_test_db() -> Result<Pool, Box<dyn std::error::Error + Send + Sync>> {
    // Ensure RUN_ENV is set to test for config loading
    std::env::set_var("RUN_ENV", "test");

    // Provide default test database environment variables if not set externally.
    // These should match the structure expected by `AppConfig` and `db::establish_connection`.
    // `db::establish_connection` uses `DatabaseConfig.url` as the database NAME.
    if std::env::var("TEST_APP_DATABASE_URL").is_err() {
        std::env::set_var("TEST_APP_DATABASE_URL", "test_db"); // Placeholder DB Name
    }
    if std::env::var("TEST_APP_DATABASE_USERNAME").is_err() {
        std::env::set_var("TEST_APP_DATABASE_USERNAME", "postgres");
    }
    if std::env::var("TEST_APP_DATABASE_PASSWORD").is_err() {
        std::env::set_var("TEST_APP_DATABASE_PASSWORD", "password");
    }
    if std::env::var("TEST_APP_DATABASE_HOST").is_err() {
        std::env::set_var("TEST_APP_DATABASE_HOST", "localhost");
    }
    if std::env::var("TEST_APP_DATABASE_PORT").is_err() {
        std::env::set_var("TEST_APP_DATABASE_PORT", "5432");
    }
    if std::env::var("TEST_APP_DATABASE_MAX_CONNECTIONS").is_err() {
        std::env::set_var("TEST_APP_DATABASE_MAX_CONNECTIONS", "10");
    }

    let app_config = AppConfig::new().map_err(|e| format!("Failed to load app config for test DB: {}", e))?;
    let db_config = app_config.database;

    // For debugging: println!("Test DB Config used by setup_test_db: {:?}", db_config);

    let pool = db::establish_connection(&db_config)
        .map_err(|e| format!("Failed to establish test DB connection (using config: {:?}): {}", db_config, e))?;

    let client = pool.get().await // Removed mut
        .map_err(|e| format!("Failed to get client from test DB pool: {}", e))?;

    // Read and execute migration SQL
    // Path relative to workspace root (where Cargo.toml is)
    let migration_file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations/20240911144657_recreate_tables.sql");
    let sql = fs::read_to_string(&migration_file_path)
        .map_err(|e| format!("Failed to read migration file {:?}: {}", migration_file_path, e))?;

    client.batch_execute(&sql).await
        .map_err(|e| format!("Failed to execute migrations: {}", e))?;

    Ok(pool)
}

/// Optional: Clears data from tables. Not implemented yet, as re-running migrations is often preferred.
#[allow(dead_code)]
pub async fn clear_tables(_client: &mut Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Example: client.batch_execute("TRUNCATE users, api_statistics RESTART IDENTITY CASCADE;").await?;
    unimplemented!("Table clearing not implemented yet. Re-run migrations via setup_test_db.");
}
