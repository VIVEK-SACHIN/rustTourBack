use std::time::Duration;

use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::utils::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub id: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn parse_jwt_expires(s: &str) -> Duration {
    let s = s.trim();
    if let Some(d) = s.strip_suffix('d') {
        if let Ok(n) = d.trim().parse::<u64>() {
            return Duration::from_secs(n * 86_400);
        }
    }
    if let Some(h) = s.strip_suffix('h') {
        if let Ok(n) = h.trim().parse::<u64>() {
            return Duration::from_secs(n * 3_600);
        }
    }
    if let Some(m) = s.strip_suffix('m') {
        if let Ok(n) = m.trim().parse::<u64>() {
            return Duration::from_secs(n * 60);
        }
    }
    if let Ok(secs) = s.parse::<u64>() {
        return Duration::from_secs(secs);
    }
    Duration::from_secs(90 * 86_400)
}

pub fn sign_jwt(user_id_hex: &str, config: &AppConfig) -> Result<String, AppError> {
    let secret = config.jwt_secret.as_bytes();
    if secret.is_empty() {
        return Err(AppError::internal("JWT_SECRET is not configured."));
    }
    let ttl = parse_jwt_expires(&config.jwt_expires_in);
    let now = Utc::now().timestamp();
    let exp = now + ttl.as_secs() as i64;
    let claims = JwtClaims {
        id: user_id_hex.to_string(),
        exp,
        iat: now,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| AppError::internal(e.to_string()))
}
