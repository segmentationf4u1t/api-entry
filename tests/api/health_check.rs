use crate::common::spawn_app; // Use the spawn_app helper from common.rs

// Note: We need an HTTP client. `reqwest` is good for external-style testing.
// Add `reqwest = { version = "0.11", features = ["json"] }` to [dev-dependencies] in Cargo.toml

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0)); // Assuming health check returns empty body
}
