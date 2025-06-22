use std::collections::HashMap;
use deadpool_postgres::Pool;
use deadpool_postgres::Client;
use crate::db;
use serde::Serialize;
use chrono::{DateTime, Utc};

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tokio::sync::RwLock;

#[derive(Clone, Serialize)]
pub struct StatisticsData {
    pub total_requests: i64,  // Changed from u64 to i64
    pub avg_response_time: f64,
    pub error_rate: f64,
    pub uptime: f64,
    pub traffic_distribution: HashMap<String, u64>,
    pub last_requests: Vec<RequestLog>,
    pub error_log: Vec<ErrorLog>,
    pub last_saved: Option<DateTime<Utc>>,
    pub register_requests: usize,
    pub register_success: usize,
    pub get_user_requests: usize,
    pub get_user_success: usize,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Serialize)]
pub struct RequestLog {
    pub method: String,
    pub endpoint: String,
    pub status: u16,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Serialize)]
pub struct ErrorLog {
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

pub struct Statistics {
    data: RwLock<StatisticsData>,
    register_requests: AtomicUsize,
    register_success: AtomicUsize,
    get_user_requests: AtomicUsize,
    get_user_success: AtomicUsize,
}

impl Statistics {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(StatisticsData {
                // Initialize with default values
                total_requests: 0,
                avg_response_time: 0.0,
                error_rate: 0.0,
                uptime: 0.0,
                traffic_distribution: HashMap::new(),
                last_requests: Vec::new(),
                error_log: Vec::new(),
                last_saved: None,
                register_requests: 0,
                register_success: 0,
                get_user_requests: 0,
                get_user_success: 0,
                timestamp: Utc::now(),
            }),
            register_requests: AtomicUsize::new(0),
            register_success: AtomicUsize::new(0),
            get_user_requests: AtomicUsize::new(0),
            get_user_success: AtomicUsize::new(0),
        }
    }

    pub async fn increment(&self, key: &str) {
        match key {
            "register_requests" => self.register_requests.fetch_add(1, Ordering::SeqCst),
            "register_success" => self.register_success.fetch_add(1, Ordering::SeqCst),
            "get_user_requests" => self.get_user_requests.fetch_add(1, Ordering::SeqCst),
            "get_user_success" => self.get_user_success.fetch_add(1, Ordering::SeqCst),
            _ => 0,
        };
    }

    pub async fn get_statistics(&self, client: &Client) -> Result<StatisticsData, Box<dyn std::error::Error>> {
        let db_stats = db::get_latest_statistics(client).await?;
        let traffic_distribution = db::get_traffic_distribution(client).await?;
        let last_requests = db::get_last_requests(client).await?;
        let error_log = db::get_error_log(client).await?;

        Ok(StatisticsData {
            total_requests: db_stats.total_requests,
            avg_response_time: db_stats.avg_response_time,
            error_rate: db_stats.error_rate,
            uptime: db_stats.uptime,
            traffic_distribution,
            last_requests,
            error_log,
            last_saved: Some(db_stats.timestamp),
            register_requests: db_stats.register_requests as usize,
            register_success: db_stats.register_success as usize,
            get_user_requests: db_stats.get_user_requests as usize,
            get_user_success: db_stats.get_user_success as usize,
            timestamp: db_stats.timestamp,
        })
    }

    pub async fn save(&self, pool: &Pool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = self.data.read().await.clone();
        save_statistics_to_db(pool, &data).await
    }

    pub async fn update_uptime(&self, uptime: f64) {
        let mut data = self.data.write().await;
        data.uptime = uptime;
    }

    pub async fn log_request(&self, _method: &str, _path: &str, _status: u16, _duration: f64) {
        let mut data = self.data.write().await;
        data.total_requests += 1;
        // Update other statistics as needed
        // For example, add to last_requests deque, update traffic_distribution, avg_response_time etc.
        // data.last_requests.push_back(RequestLog { method: _method.to_string(), endpoint: _path.to_string(), status: _status, timestamp: Utc::now() });
        // if data.last_requests.len() > SOME_LIMIT { data.last_requests.pop_front(); }
        // *data.traffic_distribution.entry(_path.to_string()).or_insert(0) += 1;
    }
}

async fn save_statistics_to_db(pool: &Pool, data: &StatisticsData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = pool.get().await?;
    db::insert_statistics(&client, data).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn test_statistics_new() {
        let stats = Statistics::new();
        assert_eq!(stats.register_requests.load(Ordering::SeqCst), 0);
        assert_eq!(stats.register_success.load(Ordering::SeqCst), 0);
        assert_eq!(stats.get_user_requests.load(Ordering::SeqCst), 0);
        assert_eq!(stats.get_user_success.load(Ordering::SeqCst), 0);

        let data = stats.data.read().await;
        assert_eq!(data.total_requests, 0);
        assert_eq!(data.uptime, 0.0);
    }

    #[tokio::test]
    async fn test_statistics_increment() {
        let stats = Statistics::new();

        stats.increment("register_requests").await;
        assert_eq!(stats.register_requests.load(Ordering::SeqCst), 1);

        stats.increment("register_success").await;
        assert_eq!(stats.register_success.load(Ordering::SeqCst), 1);

        stats.increment("get_user_requests").await;
        assert_eq!(stats.get_user_requests.load(Ordering::SeqCst), 1);

        stats.increment("get_user_success").await;
        assert_eq!(stats.get_user_success.load(Ordering::SeqCst), 1);

        // Test unknown key
        stats.increment("unknown_key").await;
        assert_eq!(stats.register_requests.load(Ordering::SeqCst), 1); // Should not change others
        assert_eq!(stats.get_user_success.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_update_uptime() {
        let stats = Statistics::new();
        let new_uptime = 99.95;
        stats.update_uptime(new_uptime).await;

        let data = stats.data.read().await;
        assert_eq!(data.uptime, new_uptime);
    }

    #[tokio::test]
    async fn test_log_request() {
        let stats = Statistics::new();

        let initial_total_requests = stats.data.read().await.total_requests;
        assert_eq!(initial_total_requests, 0);

        stats.log_request("GET", "/test", 200, 0.123).await;

        let data_after_log = stats.data.read().await;
        assert_eq!(data_after_log.total_requests, initial_total_requests + 1);

        // If other fields in StatisticsData were updated by log_request, they would be tested here.
        // For example, if avg_response_time was calculated:
        // assert_ne!(data_after_log.avg_response_time, 0.0);
    }

    // Unit tests for `get_statistics` and `save` would require mocking database interactions.
    // These are better covered by integration tests or tests with a real test database.
    // For example, a test for `save_statistics_to_db` would look like this if we had db mocking:
    //
    // use crate::tests::helpers::test_helpers; // Assuming setup_test_db is available
    // use crate::db; // For mockable db functions
    //
    // #[tokio::test]
    // async fn test_save_statistics_to_db_mocked() {
    //     // Mock setup for db::insert_statistics
    // }
}