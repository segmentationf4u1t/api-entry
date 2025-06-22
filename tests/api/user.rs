use crate::common::spawn_app;
use my_actix_api::auth::User; // Assuming User struct is needed for response deserialization
use my_actix_api::routes::user::{RegisterUserPayload, LoginUserPayload}; // Payloads for requests
use serde_json::json;

#[tokio::test]
async fn register_user_success() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let unique_email = format!("testuser_{}@example.com", chrono::Utc::now().timestamp_micros());
    let payload = RegisterUserPayload {
        email: unique_email.clone(),
        username: format!("testuser_{}", chrono::Utc::now().timestamp_micros()),
        password: "Password123!".to_string(),
    };

    let response = client
        .post(&format!("{}/users/register", &app.address))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 201, "Expected 201 Created status");

    let user_response: User = response.json().await.expect("Failed to parse user response");
    assert_eq!(user_response.email, unique_email);
    assert_eq!(user_response.username, payload.username);
    // Further checks on the user object can be added if needed
}

#[tokio::test]
async fn register_user_duplicate_email() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let unique_email = format!("duplicate_{}@example.com", chrono::Utc::now().timestamp_micros());
    let payload1 = RegisterUserPayload {
        email: unique_email.clone(),
        username: "user1_dup_email".to_string(),
        password: "Password123!".to_string(),
    };
    let payload2 = RegisterUserPayload {
        email: unique_email.clone(), // Same email
        username: "user2_dup_email".to_string(),
        password: "Password456!".to_string(),
    };

    // First registration should succeed
    let response1 = client
        .post(&format!("{}/users/register", &app.address))
        .json(&payload1)
        .send()
        .await
        .expect("Failed to execute request for first user.");
    assert_eq!(response1.status().as_u16(), 201, "First registration should succeed");

    // Second registration with the same email should fail
    let response2 = client
        .post(&format!("{}/users/register", &app.address))
        .json(&payload2)
        .send()
        .await
        .expect("Failed to execute request for second user.");

    assert_eq!(response2.status().as_u16(), 400, "Expected 400 Bad Request for duplicate email");
    let error_response: serde_json::Value = response2.json().await.expect("Failed to parse error response");
    assert!(error_response["message"].as_str().unwrap().contains("User with this email or username already exists"));
}


#[tokio::test]
async fn register_user_invalid_payload() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Example: missing email
    let payload = json!({
        "username": "invalid_payload_user",
        "password": "Password123!"
    });

    let response = client
        .post(&format!("{}/users/register", &app.address))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 400, "Expected 400 Bad Request for invalid payload");
    // Further checks on the error message can be added if needed
    let error_body: serde_json::Value = response.json().await.expect("Failed to parse error response");
    assert!(error_body["message"].to_string().to_lowercase().contains("deserialize") || // Serde error
            error_body["message"].to_string().to_lowercase().contains("invalid") || // Validation error
            error_body["message"].to_string().to_lowercase().contains("missing field"));
}

// TODO: Add tests for login (success, failure - wrong password, user not found)
// TODO: Add tests for get user by ID (success, not found, authorization if any)
// These would require a user to be registered first, and potentially a token from login.

/*
#[tokio::test]
async fn login_user_success() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let unique_email = format!("login_test_{}@example.com", chrono::Utc::now().timestamp_micros());
    let password = "LoginPassword123!";
    let register_payload = RegisterUserPayload {
        email: unique_email.clone(),
        username: format!("login_user_{}", chrono::Utc::now().timestamp_micros()),
        password: password.to_string(),
    };

    // Register user first
    let reg_response = client
        .post(&format!("{}/users/register", &app.address))
        .json(&register_payload)
        .send().await.expect("Register failed");
    assert_eq!(reg_response.status().as_u16(), 201);

    let login_payload = LoginUserPayload {
        email: unique_email,
        password: password.to_string(),
    };

    let login_response = client
        .post(&format!("{}/users/login", &app.address))
        .json(&login_payload)
        .send().await.expect("Login failed");

    assert_eq!(login_response.status().as_u16(), 200, "Login should succeed");
    let response_body: serde_json::Value = login_response.json().await.expect("Failed to parse login response");
    assert!(response_body["token"].as_str().is_some(), "Token should be present in login response");
}
*/
