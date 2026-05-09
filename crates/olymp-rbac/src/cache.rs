use redis::AsyncCommands;
use uuid::Uuid;

use crate::models::EffectivePermissions;
use crate::repository::RbacRepository;
use olymp_core::AppError;

const CACHE_PREFIX: &str = "rbac:";
const CACHE_TTL_SECS: u64 = 900; // 15 minutes

pub struct RbacCache;

impl RbacCache {
    fn key(user_id: Uuid) -> String {
        format!("{}{}", CACHE_PREFIX, user_id)
    }

    /// Get effective permissions from cache, or build from DB and cache.
    pub async fn get_or_build(
        pool: &sqlx::PgPool,
        redis: &redis::Client,
        user_id: Uuid,
    ) -> Result<EffectivePermissions, AppError> {
        // Try cache first
        if let Ok(mut conn) = redis.get_multiplexed_async_connection().await {
            let cached: Result<String, _> = conn.get(Self::key(user_id)).await;
            if let Ok(json) = cached {
                if let Ok(perms) = serde_json::from_str::<EffectivePermissions>(&json) {
                    return Ok(perms);
                }
            }
        }

        // Cache miss → build from DB
        let perms = RbacRepository::build_effective_permissions(pool, user_id).await?;

        // Cache it (best-effort)
        if let Ok(mut conn) = redis.get_multiplexed_async_connection().await {
            if let Ok(json) = serde_json::to_string(&perms) {
                let _: Result<(), _> = conn
                    .set_ex(Self::key(user_id), json, CACHE_TTL_SECS)
                    .await;
            }
        }

        Ok(perms)
    }

    /// Invalidate cache for a user (call on assignment/role/permission changes).
    pub async fn invalidate(redis: &redis::Client, user_id: Uuid) {
        if let Ok(mut conn) = redis.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.del(Self::key(user_id)).await;
        }
    }

    /// Invalidate all users with given role (expensive, use sparingly).
    pub async fn invalidate_role(
        pool: &sqlx::PgPool,
        redis: &redis::Client,
        role_id: Uuid,
    ) {
        if let Ok(user_ids) = sqlx::query_scalar::<_, Uuid>(
            "SELECT DISTINCT user_id FROM user_role_assignments WHERE role_id = $1 AND is_active = true",
        )
        .bind(role_id)
        .fetch_all(pool)
        .await
        {
            for uid in user_ids {
                Self::invalidate(redis, uid).await;
            }
        }
    }
}
