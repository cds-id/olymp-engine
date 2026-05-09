use olymp_core::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct OAuthService {
    pool: PgPool,
}

impl OAuthService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Link OAuth provider to user
    pub async fn link_provider(
        &self,
        user_id: Uuid,
        provider: &str,
        provider_user_id: &str,
        provider_email: Option<&str>,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO auth.oauth_providers (id, user_id, provider, provider_user_id, provider_email, access_token, refresh_token) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (provider, provider_user_id) DO UPDATE SET access_token = $6, refresh_token = $7, updated_at = now()"
        )
        .bind(Uuid::now_v7())
        .bind(user_id)
        .bind(provider)
        .bind(provider_user_id)
        .bind(provider_email)
        .bind(access_token)
        .bind(refresh_token)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(())
    }

    /// Get user by OAuth provider
    pub async fn get_user_by_provider(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<Option<Uuid>, AppError> {
        let result = sqlx::query_scalar::<_, Uuid>(
            "SELECT user_id FROM auth.oauth_providers WHERE provider = $1 AND provider_user_id = $2"
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e))?;

        Ok(result)
    }

    /// Unlink OAuth provider
    pub async fn unlink_provider(&self, user_id: Uuid, provider: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM auth.oauth_providers WHERE user_id = $1 AND provider = $2")
            .bind(user_id)
            .bind(provider)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }
}
