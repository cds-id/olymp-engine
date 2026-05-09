use blurp_core::error::AppError;
use chrono::{Duration, Utc};
use sha2::{Sha256, Digest};
use hex;
use sqlx::PgPool;
use uuid::Uuid;

pub struct MagicLinkService {
    pool: PgPool,
    ttl_secs: u64,
}

impl MagicLinkService {
    pub fn new(pool: PgPool, ttl_secs: u64) -> Self {
        Self { pool, ttl_secs }
    }

    pub async fn create(&self, email: &str) -> Result<String, AppError> {
        let token = Uuid::now_v7().to_string();
        let token_hash = self.hash_token(&token);
        let expires_at = Utc::now() + Duration::seconds(self.ttl_secs as i64);

        sqlx::query(
            "INSERT INTO auth.magic_links (email, token_hash, expires_at) 
             VALUES ($1, $2, $3)
             ON CONFLICT (email) DO UPDATE SET token_hash = $2, expires_at = $3, used_at = NULL"
        )
        .bind(email)
        .bind(&token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(token)
    }

    pub async fn verify(&self, email: &str, token: &str) -> Result<bool, AppError> {
        let token_hash = self.hash_token(token);
        let result = sqlx::query_scalar::<_, i32>(
            "UPDATE auth.magic_links 
             SET used_at = now() 
             WHERE email = $1 AND token_hash = $2 AND expires_at > now() AND used_at IS NULL
             RETURNING 1"
        )
        .bind(email)
        .bind(&token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(result.is_some())
    }

    fn hash_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
}
