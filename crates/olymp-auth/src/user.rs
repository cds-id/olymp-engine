use olymp_core::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub auth_method: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_or_create(&self, email: &str) -> Result<Uuid, AppError> {
        let user_id = sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT id FROM auth.users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?
        .flatten();

        if let Some(id) = user_id {
            return Ok(id);
        }

        let new_id = Uuid::now_v7();
        sqlx::query(
            "INSERT INTO auth.users (id, email, name, is_guest) VALUES ($1, $2, $3, false)"
        )
        .bind(new_id)
        .bind(email)
        .bind(email.split('@').next().unwrap_or("User"))
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(new_id)
    }

    pub async fn get_by_id(&self, user_id: Uuid) -> Result<Option<(Uuid, String)>, AppError> {
        sqlx::query_as::<_, (Uuid, String)>("SELECT id, email FROM auth.users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))
    }

    pub async fn get_full_by_id(&self, user_id: Uuid) -> Result<Option<UserProfile>, AppError> {
        sqlx::query_as::<_, UserProfile>(
            "SELECT id, email, username, name, phone, auth_method, created_at, updated_at
             FROM auth.users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        name: Option<&str>,
        username: Option<&str>,
        phone: Option<&str>,
    ) -> Result<UserProfile, AppError> {
        if let Some(uname) = username {
            crate::RegistrationService::validate_username(uname)?;
            let existing = sqlx::query_scalar::<_, Uuid>(
                "SELECT id FROM auth.users WHERE username = $1 AND id != $2"
            )
            .bind(uname)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(AppError::Database)?;
            if existing.is_some() {
                return Err(AppError::Conflict("Username already taken".to_string()));
            }
        }

        sqlx::query(
            "UPDATE auth.users SET
               name = COALESCE($1, name),
               username = COALESCE($2, username),
               phone = COALESCE($3, phone),
               updated_at = now()
             WHERE id = $4"
        )
        .bind(name)
        .bind(username)
        .bind(phone)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        self.get_full_by_id(user_id)
            .await?
            .ok_or(AppError::NotFound("User not found".to_string()))
    }
}
