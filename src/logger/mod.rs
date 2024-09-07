use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use chrono::Utc;
use futures::future::{ok, Ready};
use futures::Future;
use log::{error, info};
use std::pin::Pin;
use std::task::{Context, Poll};

// Logger struct for creating the middleware
pub struct Logger;

// Implementation of the Transform trait for Logger
impl<S, B> Transform<S, ServiceRequest> for Logger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    // Create a new LoggerMiddleware
    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggerMiddleware { service })
    }
}

// LoggerMiddleware struct that wraps the inner service
pub struct LoggerMiddleware<S> {
    service: S,
}

// Implementation of the Service trait for LoggerMiddleware
impl<S, B> Service<ServiceRequest> for LoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    // Check if the service is ready
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    // Handle the incoming request
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Utc::now();
        let method = req.method().to_string();
        let path = req.path().to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let elapsed = Utc::now().signed_duration_since(start_time).num_milliseconds();

            // Create log message
            let log_message = format!(
                "Request processed - Timestamp: {}, Method: {}, Path: {}, Status: {}, Elapsed: {}ms",
                start_time, method, path, res.status(), elapsed
            );

            // Log based on response status
            if res.status().is_server_error() {
                error!(target: "my_actix_api", "{}", log_message);
            } else {
                info!(target: "my_actix_api", "{}", log_message);
            }

            Ok(res)
        })
    }
}