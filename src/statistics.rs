use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use deadpool_postgres::Pool;
use log::{error, info};
use crate::db;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct StatisticsData {
    pub data: HashMap<String, i64>,
}

pub struct Statistics {
    data: Arc<Mutex<HashMap<String, i64>>>,
    pool: Pool,
}

impl Statistics {
    pub fn new(pool: Pool) -> Self {
        Statistics {
            data: Arc::new(Mutex::new(HashMap::new())),
            pool,
        }
    }

    pub async fn increment(&self, key: &str) {
        let mut data = self.data.lock().await;
        *data.entry(key.to_string()).or_insert(0) += 1;
    }

    pub async fn get_statistics(&self) -> StatisticsData {
        let data = self.data.lock().await.clone();
        StatisticsData { data }
    }

    pub async fn save(&self) {
        let data = self.data.lock().await.clone();
        if data.is_empty() {
            return;
        }

        let client = match self.pool.get().await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to get database connection: {}", e);
                return;
            }
        };

        if let Err(e) = db::insert_statistics(&client, &data).await {
            error!("Failed to insert statistics: {}", e);
        } else {
            info!("Statistics saved successfully");
        }

        // Clear the data after saving
        self.data.lock().await.clear();
    }
}