use crate::common::spawn_app;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn rate_limit_blocks_excessive_requests() {
    let app = spawn_app().await; // spawn_app configures RateLimiter based on AppConfig
    let client = reqwest::Client::new();

    // Need to know the rate limit settings from AppConfig.
    // Let's assume example.config.toml has:
    // requests_per_second = 5 (or some known value)
    // burst_size = 10 (or some known value)
    // For this test, let's assume a low limit for easier testing, e.g., 2 req/sec, burst 2.
    // If the actual config is higher, this test might need adjustment or a way to
    // override config for this specific test run (which is more complex).

    // The `spawn_app` uses AppConfig::new(), which loads from `config.toml` or `example.config.toml`.
    // Let's check `example.config.toml` to understand the default rate limits.
    // If `example.config.toml` has high limits, this test will be slow or impractical.
    // For now, let's assume the limits are reasonably low for testing, e.g., 5 req/sec, burst 5.
    // We will make more requests than burst_size quickly.

    let client = reqwest::Client::new();

    // From example.config.toml: burst_size = 3, requests_per_second = 50
    let configured_burst_size = 3;
    let requests_to_send = configured_burst_size + 2; // Send a few more than burst size

    let target_url = format!("{}/health", &app.address); // Use a simple, fast endpoint
    let mut received_429 = false;

    println!("Sending {} requests to {} with configured burst_size = {}", requests_to_send, target_url, configured_burst_size);

    for i in 0..requests_to_send {
        let response = client.get(&target_url).send().await.expect("Request failed");
        println!("Request {}: Status {}", i + 1, response.status());

        if response.status().as_u16() == 429 {
            received_429 = true;
            // break; // We can break once we confirm rate limiting, or continue to see subsequent behavior
        } else if !response.status().is_success() {
            // Handle other unexpected errors
            panic!("Unexpected status code: {} for request {}", response.status(), i + 1);
        }
        // No sleep, send requests as fast as possible to hit burst limit.
        // The governor crate handles this internally based on its clock.
    }

    assert!(received_429, "Expected to receive at least one 429 Too Many Requests status after exceeding burst limit of {}.", configured_burst_size);

    // Test that after waiting, requests are allowed again.
    // requests_per_second = 50 means 1 cell replenishes every 1000ms / 50 = 20ms.
    // To be safe and allow for some replenishment (e.g. the whole burst capacity)
    let wait_time_ms = (1000.0 / 50.0 * configured_burst_size as f64).ceil() as u64 + 50; // wait for full burst + a bit
    println!("Waiting for {}ms for rate limiter to allow requests again...", wait_time_ms);
    sleep(Duration::from_millis(wait_time_ms)).await;

    let response_after_wait = client.get(&target_url).send().await.expect("Request after wait failed");
    assert!(response_after_wait.status().is_success(),
            "Request after waiting {}ms should succeed. Status: {}", wait_time_ms, response_after_wait.status());

    // Verify a few more can go through now, up to burst size again
    for i in 0..configured_burst_size {
        let response = client.get(&target_url).send().await.expect("Request failed");
        assert!(response.status().is_success(),
            "Request {} after wait should succeed. Status: {}", i, response.status());
    }

    // And one more should ideally fail if we exhaust the burst again quickly
    let response_after_burst_refill = client.get(&target_url).send().await.expect("Request failed");
    if configured_burst_size > 0 { // only makes sense if burst > 0
        assert_eq!(response_after_burst_refill.status().as_u16(), 429,
            "Request after exhausting refilled burst should be rate limited again.");
    }
}
