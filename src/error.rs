use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;
use thiserror::Error;

#[allow(dead_code)] // Add this line to suppress dead code warnings for the entire enum
#[derive(Error, Debug)]
pub enum AppError {
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

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
    pub error_type: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error_type: format!("{:?}", self),
        };
        HttpResponse::build(status_code).json(error_response)
    }

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

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ \"code\": {}, \"message\": \"{}\", \"error_type\": \"{}\" }}",
            self.code, self.message, self.error_type
        )
    }
}