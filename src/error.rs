use actix_web::{http::StatusCode, ResponseError}; // Removed HttpResponse
use serde::Serialize;
use std::fmt;
use thiserror::Error;

// Define custom error types for the application
// The #[allow(dead_code)] attribute suppresses warnings for unused enum variants
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum AppError {
    // Define various error types with associated error messages
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Not Found")]
    NotFound,
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Rate Limit Exceeded")]
    RateLimitExceeded,
    #[error("Database error: {0}")]
    DatabaseError(String),
}

// Define the structure for error responses
// This will be serialized to JSON when sent to the client
#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub error_type: String,
}

// Implement ResponseError trait for AppError
// This allows our custom error type to be used with actix-web
impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR, // Add this line
        }
    }
}

// Implement Display trait for ErrorResponse
// This allows the ErrorResponse to be easily converted to a string
impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ \"code\": {}, \"message\": \"{}\", \"error_type\": \"{}\" }}",
            self.code, self.message, self.error_type
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    // Test the status_code method for each AppError variant
    #[test]
    fn test_app_error_status_codes() {
        assert_eq!(AppError::InternalServerError.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(AppError::NotFound.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(AppError::BadRequest("test".to_string()).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(AppError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(AppError::RateLimitExceeded.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(AppError::DatabaseError("db issue".to_string()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Test the Display trait implementation (derived by thiserror) for AppError
    #[test]
    fn test_app_error_display() {
        assert_eq!(AppError::InternalServerError.to_string(), "Internal Server Error");
        assert_eq!(AppError::NotFound.to_string(), "Not Found");
        assert_eq!(AppError::BadRequest("test message".to_string()).to_string(), "Bad Request: test message");
        assert_eq!(AppError::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(AppError::RateLimitExceeded.to_string(), "Rate Limit Exceeded");
        assert_eq!(AppError::DatabaseError("connection failed".to_string()).to_string(), "Database error: connection failed");
    }

    // Test the Display trait implementation for ErrorResponse
    #[test]
    fn test_error_response_display() {
        let error_response = ErrorResponse {
            code: 404,
            message: "Resource not found".to_string(),
            error_type: "NotFound".to_string(),
        };
        assert_eq!(
            error_response.to_string(),
            r#"{ "code": 404, "message": "Resource not found", "error_type": "NotFound" }"#
        );
    }

    // Test that ErrorResponse can be serialized (implicitly tested by its usage in handlers, but good to have a direct test)
    #[test]
    fn test_error_response_serialization() {
        let error_response = ErrorResponse {
            code: 500,
            message: "Server issue".to_string(),
            error_type: "InternalServerError".to_string(),
        };
        let serialized = serde_json::to_string(&error_response).unwrap();
        // Basic check, could be more specific if needed
        assert!(serialized.contains("\"code\":500"));
        assert!(serialized.contains("\"message\":\"Server issue\""));
        assert!(serialized.contains("\"error_type\":\"InternalServerError\""));
    }
}