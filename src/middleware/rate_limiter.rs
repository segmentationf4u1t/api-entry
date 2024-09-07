use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use futures::future::{ok, Ready};
use futures::Future;
use governor::{Quota, RateLimiter as GovernorRateLimiter, clock::DefaultClock};
use nonzero_ext::nonzero;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use log::{info, warn};
use chrono::Utc;
use governor::clock::Clock;
use crate::error::AppError;
use std::num::NonZeroU32;

// RateLimiter struct that holds the GovernorRateLimiter
#[derive(Clone)]
pub struct RateLimiter {
    limiter: Arc<GovernorRateLimiter<String, governor::state::keyed::DashMapStateStore<String>, DefaultClock>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        // Create a default rate limiter with 10 requests per second
        let quota = Quota::per_second(nonzero!(10u32));
        RateLimiter {
            limiter: Arc::new(GovernorRateLimiter::keyed(quota)),
        }
    }
}

impl RateLimiter {
    // Create a new RateLimiter with specified requests per second and burst size
    pub fn new(requests_per_second: u32, burst_size: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap())
            .allow_burst(NonZeroU32::new(burst_size).unwrap());
        RateLimiter {
            limiter: Arc::new(GovernorRateLimiter::keyed(quota)),
        }
    }
}

// Implement Transform trait for RateLimiter
impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    // Create a new RateLimiterMiddleware
    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterMiddleware {
            service,
            limiter: self.limiter.clone(),
        })
    }
}

// RateLimiterMiddleware struct that wraps the inner service
pub struct RateLimiterMiddleware<S> {
    service: S,
    limiter: Arc<GovernorRateLimiter<String, governor::state::keyed::DashMapStateStore<String>, DefaultClock>>,
}

// Implement Service trait for RateLimiterMiddleware
impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
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
        let ip = req.peer_addr().map(|addr| addr.ip().to_string()).unwrap_or_else(|| "unknown".to_string());
        let path = req.path().to_string();
        let method = req.method().to_string();
        let timestamp = Utc::now();

        let fut = self.service.call(req);
        let limiter = self.limiter.clone();

        Box::pin(async move {
            // Check if the request is allowed by the rate limiter
            match limiter.check_key(&ip) {
                Ok(_) => {
                    // Log allowed request
                    info!(
                        target: "rate_limiter",
                        "Request allowed - Timestamp: {}, IP: {}, Method: {}, Path: {}",
                        timestamp, ip, method, path
                    );
                    fut.await
                },
                Err(negative) => {
                    // Calculate wait time and log rate limit exceeded
                    let wait_time = negative.wait_time_from(DefaultClock::default().now());
                    warn!(
                        target: "rate_limiter",
                        "Rate limit exceeded - Timestamp: {}, IP: {}, Method: {}, Path: {}, Wait time: {:?}",
                        timestamp, ip, method, path, wait_time
                    );
                    Err(AppError::RateLimitExceeded.into())
                }
            }
        })
    }
}