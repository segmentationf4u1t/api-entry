use actix_web::{http::StatusCode, HttpResponse, ResponseError};
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
    // Generate an HTTP response for each error type
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error_type: format!("{:?}", self),
        };
        HttpResponse::build(status_code).json(error_response)
    }

    // Map AppError variants to HTTP status codes
    fn status_code(&self) -> StatusCode {
        match *self {
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
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