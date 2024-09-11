use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use deadpool_postgres::Pool;
use log::{error, info};
use crate::db;
use serde::Serialize;
use chrono::{DateTime, Utc};

use crate::error::AppError;

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

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

use std::sync::RwLock;

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

    pub async fn get_statistics(&self) -> StatisticsData {
        StatisticsData {
            total_requests: (self.register_requests.load(Ordering::SeqCst) + self.get_user_requests.load(Ordering::SeqCst)) as i64,  // Cast to i64
            avg_response_time: 0.0, // Implement actual calculation
            error_rate: 1.0 - ((self.register_success.load(Ordering::SeqCst) + self.get_user_success.load(Ordering::SeqCst)) as f64 / 
                               (self.register_requests.load(Ordering::SeqCst) + self.get_user_requests.load(Ordering::SeqCst)) as f64),
            uptime: 0.0, // Implement actual uptime calculation
            traffic_distribution: HashMap::new(), // Populate with actual data
            last_requests: Vec::new(), // Populate with actual data
            error_log: Vec::new(), // Populate with actual data
            last_saved: None,
            register_requests: self.register_requests.load(Ordering::SeqCst),
            register_success: self.register_success.load(Ordering::SeqCst),
            get_user_requests: self.get_user_requests.load(Ordering::SeqCst),
            get_user_success: self.get_user_success.load(Ordering::SeqCst),
        }
    }

    pub async fn save(&self, pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
        let client = pool.get().await?;
        let stats = self.get_statistics().await;
        db::insert_statistics(&client, &stats).await?;
        Ok(())
    }

    pub fn update_uptime(&self, uptime: f64) {
        let mut data = self.data.write().unwrap();
        data.uptime = uptime;
    }

    pub fn log_request(&self, method: &str, path: &str, status: u16, duration: f64) {
        let mut data = self.data.write().unwrap();
        data.total_requests += 1;
        data.avg_response_time = (data.avg_response_time * (data.total_requests - 1) as f64 + duration) / data.total_requests as f64;
        
        let key = format!("{}_{}", method, path);
        *data.traffic_distribution.entry(key).or_insert(0) += 1;

        data.last_requests.push(RequestLog {
            method: method.to_string(),
            endpoint: path.to_string(),
            status,
            timestamp: Utc::now(),
        });

        if status >= 400 {
            data.error_log.push(ErrorLog {
                message: format!("{} {} - {}", method, path, status),
                timestamp: Utc::now(),
            });
        }
    }
}