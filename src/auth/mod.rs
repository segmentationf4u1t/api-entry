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

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, DecodingKey, Validation};

    #[test]
    fn test_hash_password_success() {
        let password = "test_password";
        let hashed_result = hash_password(password);
        assert!(hashed_result.is_ok());
        let hashed_password = hashed_result.unwrap();
        assert!(!hashed_password.is_empty());
        assert_ne!(password, hashed_password); // Hash should not be the same as password
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "test_password123";
        let hashed_password = hash_password(password).expect("Failed to hash password");
        let verification_result = verify_password(password, &hashed_password);
        assert!(verification_result.is_ok());
        assert!(verification_result.unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "test_password456";
        let wrong_password = "wrong_password789";
        let hashed_password = hash_password(password).expect("Failed to hash password");
        let verification_result = verify_password(wrong_password, &hashed_password);
        assert!(verification_result.is_ok());
        assert!(!verification_result.unwrap());
    }

    #[test]
    fn test_generate_token_success() {
        let user_id = "user123";
        let token_result = generate_token(user_id);
        assert!(token_result.is_ok());
        let token = token_result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_generate_token_payload_and_decode() {
        let user_id = "test_user_sub";
        let token_result = generate_token(user_id);
        assert!(token_result.is_ok());
        let token = token_result.unwrap();

        // Decode the token to verify its contents
        let decoding_key = DecodingKey::from_secret("your_secret_key".as_bytes());
        let mut validation = Validation::default();
        // In a real scenario, you might want to validate 'exp', but for this test,
        // we're primarily concerned with the 'sub' claim.
        // If the token is generated with default leeway, this should pass.
        // Disabling time validation for simplicity if it causes flaky tests due to exact timing.
        validation.validate_exp = true;
        // If tests are flaky due to timing, consider adding leeway or disabling exp validation for this specific test
        // validation.leeway = 5; // Allow 5 seconds leeway for expiration

        let decoded_token = decode::<Claims>(&token, &decoding_key, &validation);
        assert!(decoded_token.is_ok());
        let claims = decoded_token.unwrap().claims;
        assert_eq!(claims.sub, user_id);

        // Check that expiration is in the future (roughly)
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(claims.exp > current_timestamp as usize);
    }

    #[test]
    fn test_verify_password_with_invalid_hash_format() {
        let password = "test_password";
        let invalid_hash = "not_a_real_hash";
        let verification_result = verify_password(password, invalid_hash);
        // bcrypt verify function returns an error for malformed hashes
        assert!(verification_result.is_err());
    }
}