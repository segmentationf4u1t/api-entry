use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub avatar: Option<String>,
    pub tokens: Option<Value>,
    pub status: String,
    pub permissions: Option<Value>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    exp: usize,
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    verify(password, hash)
}

pub fn generate_token(user_id: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 24 * 3600; // 24 hours from now

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration as usize,
    };

    let header = Header::default();
    let key = EncodingKey::from_secret("your_secret_key".as_bytes());

    encode(&header, &claims, &key)
}