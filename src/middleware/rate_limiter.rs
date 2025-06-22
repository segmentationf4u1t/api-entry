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

#[cfg(test)]
mod tests {
    use super::*;
    use governor::clock::DefaultClock; // Using DefaultClock
    use governor::state::keyed::DashMapStateStore;
    use std::time::Duration;
    use std::num::NonZeroU32;
    use tokio::time::sleep; // For async sleep

    // Type alias for the specific keyed rate limiter we're testing
    type KeyedLimiter = GovernorRateLimiter<String, DashMapStateStore<String>, DefaultClock>;

    #[test]
    fn test_rate_limiter_default_creation() {
        let _limiter = RateLimiter::default();
        assert!(true, "Default creation should not panic");
    }

    #[test]
    fn test_rate_limiter_new_creation() {
        let _limiter = RateLimiter::new(5, 10);
        assert!(true, "New creation should not panic");
    }

    #[tokio::test] // Marked as async
    async fn test_rate_limiter_behavior_allows_and_blocks() {
        let quota = Quota::per_second(NonZeroU32::new(1).unwrap()).allow_burst(NonZeroU32::new(1).unwrap());
        let clock = DefaultClock::default();
        let governor_limiter = KeyedLimiter::new(quota, DashMapStateStore::default(), &clock);
        let limiter_arc = Arc::new(governor_limiter);

        let ip_key = "127.0.0.1".to_string();

        assert!(limiter_arc.check_key(&ip_key).is_ok(), "First request should be allowed");
        assert!(limiter_arc.check_key(&ip_key).is_err(), "Second request should be blocked");

        sleep(Duration::from_secs(1)).await; // Use tokio::time::sleep
        assert!(limiter_arc.check_key(&ip_key).is_ok(), "Third request after waiting should be allowed");
        assert!(limiter_arc.check_key(&ip_key).is_err(), "Fourth request should be blocked again");
    }

    #[tokio::test] // Marked as async
    async fn test_rate_limiter_burst_behavior() {
        let quota = Quota::per_second(NonZeroU32::new(2).unwrap()).allow_burst(NonZeroU32::new(3).unwrap());
        let clock = DefaultClock::default();
        let governor_limiter = KeyedLimiter::new(quota, DashMapStateStore::default(), &clock);
        let limiter_arc = Arc::new(governor_limiter);

        let ip_key = "192.168.0.1".to_string();

        // Consume initial burst
        assert!(limiter_arc.check_key(&ip_key).is_ok(), "Burst Req 1");
        assert!(limiter_arc.check_key(&ip_key).is_ok(), "Burst Req 2");
        assert!(limiter_arc.check_key(&ip_key).is_ok(), "Burst Req 3");
        assert!(limiter_arc.check_key(&ip_key).is_err(), "Burst Req 4 (should fail)");

        // Test replenishment (rate is 2rps = 500ms per cell)
        // Wait just under a cell's replenishment time
        sleep(Duration::from_millis(480)).await;
        assert!(limiter_arc.check_key(&ip_key).is_err(), "Req after ~480ms (should still be blocked)");

        // Wait a bit more, enough for one cell to certainly replenish (total ~530ms from last check)
        sleep(Duration::from_millis(50)).await; // Total sleep = 480 + 50 = 530ms
        assert!(limiter_arc.check_key(&ip_key).is_ok(), "Req after ~530ms (1 cell should have replenished)");
        // This cell is now consumed, next immediate one should fail
        assert!(limiter_arc.check_key(&ip_key).is_err(), "Req immediately after consuming the single replenished cell (should fail)");

        // Wait for 2 more cells to replenish (1 second) + buffer
        sleep(Duration::from_millis(1000 + 50)).await;
        assert!(limiter_arc.check_key(&ip_key).is_ok(), "Req after ~1s more (cell 1/2 replenished this interval)");
        assert!(limiter_arc.check_key(&ip_key).is_ok(), "Req after ~1s more (cell 2/2 replenished this interval)");
        assert!(limiter_arc.check_key(&ip_key).is_err(), "Req after ~1s more (both replenished cells consumed, should fail)");

        // Wait significantly to allow full burst refill (3 cells * 500ms/cell = 1.5s. Wait 2s.)
        sleep(Duration::from_secs(2)).await;

        // Consume refilled burst
        for i in 0..3 {
            assert!(limiter_arc.check_key(&ip_key).is_ok(),
                    "Request {} consuming refilled burst should pass", i + 1);
        }

        // Subsequent requests should hit the rate limit.
        let mut blocked_after_refill = false;
        for _ in 0..3 { // Try a few more times, i is not used
            if limiter_arc.check_key(&ip_key).is_err() {
                blocked_after_refill = true;
                break;
            }
            sleep(Duration::from_millis(50)).await; // Small delay to allow time to pass if first check was borderline
        }
        assert!(blocked_after_refill,
                "Expected at least one request to be blocked after exhausting refilled burst.");
    }

    // Note: Testing the full Actix middleware Service/Transform traits (poll_ready, call with ServiceRequest)
    // is more complex and would typically involve setting up a test Actix service.
    // The tests above focus on the core rate-limiting logic provided by the governor instance,
    // which is the heart of this middleware.
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