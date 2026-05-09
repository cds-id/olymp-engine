use blurp_core::error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct NotificationPreferences {
    pub id: Uuid,
    pub user_id: Uuid,
    pub order_updates: bool,
    pub promotions: bool,
    pub security_alerts: bool,
    pub newsletter: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateNotificationPreferences {
    pub order_updates: Option<bool>,
    pub promotions: Option<bool>,
    pub security_alerts: Option<bool>,
    pub newsletter: Option<bool>,
}

pub struct NotificationPrefsService {
    pool: PgPool,
}

impl NotificationPrefsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get or create default preferences
    pub async fn get(&self, user_id: Uuid) -> Result<NotificationPreferences, AppError> {
        let prefs = sqlx::query_as::<_, NotificationPreferences>(
            "SELECT * FROM auth.notification_preferences WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::Database)?;

        match prefs {
            Some(p) => Ok(p),
            None => {
                // Create defaults
                sqlx::query(
                    "INSERT INTO auth.notification_preferences (user_id) VALUES ($1)"
                )
                .bind(user_id)
                .execute(&self.pool)
                .await
                .map_err(AppError::Database)?;

                sqlx::query_as::<_, NotificationPreferences>(
                    "SELECT * FROM auth.notification_preferences WHERE user_id = $1"
                )
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(AppError::Database)
            }
        }
    }

    pub async fn update(
        &self,
        user_id: Uuid,
        req: UpdateNotificationPreferences,
    ) -> Result<NotificationPreferences, AppError> {
        // Ensure row exists
        self.get(user_id).await?;

        sqlx::query(
            "UPDATE auth.notification_preferences SET
               order_updates = COALESCE($1, order_updates),
               promotions = COALESCE($2, promotions),
               security_alerts = COALESCE($3, security_alerts),
               newsletter = COALESCE($4, newsletter),
               updated_at = now()
             WHERE user_id = $5"
        )
        .bind(req.order_updates)
        .bind(req.promotions)
        .bind(req.security_alerts)
        .bind(req.newsletter)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        self.get(user_id).await
    }
}
