use olymp_core::error::AppError;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct PasswordService {
    pool: PgPool,
}

impl PasswordService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Hash password with argon2
    pub fn hash_password(password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(rand::thread_rng());
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AppError::BadRequest("Failed to hash password".to_string()))?
            .to_string();
        Ok(hash)
    }

    /// Verify password against hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| AppError::BadRequest("Invalid password hash".to_string()))?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Validate password strength
    pub fn validate_password(password: &str) -> Result<(), AppError> {
        if password.len() < 8 {
            return Err(AppError::BadRequest("Password must be at least 8 characters".to_string()));
        }
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(AppError::BadRequest("Password must contain uppercase letter".to_string()));
        }
        if !password.chars().any(|c| c.is_numeric()) {
            return Err(AppError::BadRequest("Password must contain number".to_string()));
        }
        Ok(())
    }

    /// Create password reset token
    pub async fn create_reset_token(&self, user_id: Uuid) -> Result<String, AppError> {
        let token = Uuid::now_v7().to_string();
        let expires_at = Utc::now() + Duration::hours(1);

        sqlx::query(
            "INSERT INTO auth.password_resets (id, user_id, token_hash, expires_at) 
             VALUES ($1, $2, $3, $4)"
        )
        .bind(Uuid::now_v7())
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(token)
    }

    /// Verify reset token and return user_id
    pub async fn verify_reset_token(&self, token: &str) -> Result<Uuid, AppError> {
        let result = sqlx::query_scalar::<_, Uuid>(
            "UPDATE auth.password_resets 
             SET used_at = now() 
             WHERE token_hash = $1 AND expires_at > now() AND used_at IS NULL
             RETURNING user_id"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        result.ok_or(AppError::BadRequest("Invalid or expired reset token".to_string()))
    }

    /// Update user password
    pub async fn update_password(&self, user_id: Uuid, new_password: &str) -> Result<(), AppError> {
        Self::validate_password(new_password)?;
        let hash = Self::hash_password(new_password)?;

        sqlx::query("UPDATE auth.users SET password_hash = $1, updated_at = now() WHERE id = $2")
            .bind(&hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }
}
