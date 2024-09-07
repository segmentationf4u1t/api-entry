use deadpool_postgres::Client;
use tokio_postgres::Error as PgError;

pub async fn create_user_table(client: &Client) -> Result<(), PgError> {
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                username VARCHAR(50) UNIQUE NOT NULL,
                password VARCHAR(100) NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
            )",
            &[],
        )
        .await?;
    Ok(())
}

pub async fn insert_user(client: &Client, username: &str, password: &str) -> Result<(), PgError> {
    client
        .execute(
            "INSERT INTO users (username, password) VALUES ($1, $2)",
            &[&username, &password],
        )
        .await?;
    Ok(())
}

pub async fn get_user_by_username(client: &Client, username: &str) -> Result<Option<(i32, String, String)>, PgError> {
    let row = client
        .query_opt("SELECT id, username, password FROM users WHERE username = $1", &[&username])
        .await?;
    
    Ok(row.map(|r| (r.get(0), r.get(1), r.get(2))))
}