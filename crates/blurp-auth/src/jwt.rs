use blurp_core::error::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub jti: String, // unique token ID for revocation
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshClaims {
    pub sub: String,
    pub token_type: String, // "refresh"
    pub iat: i64,
    pub exp: i64,
}

pub struct JwtService {
    secret: String,
    access_ttl_secs: u64,
    refresh_ttl_secs: u64,
}

impl JwtService {
    pub fn new(secret: String, access_ttl_secs: u64) -> Self {
        Self {
            secret,
            access_ttl_secs,
            refresh_ttl_secs: 604800, // 7 days
        }
    }

    pub fn issue(&self, user_id: Uuid, email: String) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.access_ttl_secs as i64);

        let claims = Claims {
            sub: user_id.to_string(),
            email,
            jti: Uuid::now_v7().to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub fn verify(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized)
    }

    pub fn issue_refresh(&self, user_id: Uuid) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.refresh_ttl_secs as i64);
        let claims = RefreshClaims {
            sub: user_id.to_string(),
            token_type: "refresh".to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub fn verify_refresh(&self, token: &str) -> Result<RefreshClaims, AppError> {
        let data = decode::<RefreshClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        if data.claims.token_type != "refresh" {
            return Err(AppError::Unauthorized);
        }
        Ok(data.claims)
    }
}
