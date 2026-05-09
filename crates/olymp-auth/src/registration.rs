use olymp_core::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::password::PasswordService;

pub struct RegistrationService {
    pool: PgPool,
}

impl RegistrationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Validate username
    pub fn validate_username(username: &str) -> Result<(), AppError> {
        if username.len() < 3 {
            return Err(AppError::BadRequest("Username must be at least 3 characters".to_string()));
        }
        if username.len() > 50 {
            return Err(AppError::BadRequest("Username must be at most 50 characters".to_string()));
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(AppError::BadRequest("Username can only contain alphanumeric, _, -".to_string()));
        }
        Ok(())
    }

    /// Register with username + password
    pub async fn register_with_password(
        &self,
        email: &str,
        username: &str,
        password: &str,
        name: Option<&str>,
    ) -> Result<Uuid, AppError> {
        Self::validate_username(username)?;
        PasswordService::validate_password(password)?;

        let password_hash = PasswordService::hash_password(password)?;
        let user_id = Uuid::now_v7();

        sqlx::query(
            "INSERT INTO auth.users (id, email, username, password_hash, name, auth_method) 
             VALUES ($1, $2, $3, $4, $5, 'password')"
        )
        .bind(user_id)
        .bind(email)
        .bind(username)
        .bind(&password_hash)
        .bind(name.unwrap_or(username))
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") {
                if e.to_string().contains("email") {
                    AppError::BadRequest("Email already registered".to_string())
                } else {
                    AppError::BadRequest("Username already taken".to_string())
                }
            } else {
                AppError::Database(e)
            }
        })?;

        Ok(user_id)
    }

    /// Get user by username
    pub async fn get_by_username(&self, username: &str) -> Result<Option<(Uuid, String)>, AppError> {
        sqlx::query_as::<_, (Uuid, String)>(
            "SELECT id, password_hash FROM auth.users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))
    }
}
