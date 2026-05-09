use uuid::Uuid;

use crate::cache::RbacCache;
use crate::models::{EffectivePermissions, ResourceScope};
use olymp_core::AppError;

/// RBAC context resolved for a request. Contains effective permissions.
#[derive(Debug, Clone)]
pub struct RbacContext {
    pub user_id: Uuid,
    pub permissions: EffectivePermissions,
}

impl RbacContext {
    /// Resolve RBAC context for a user (from cache or DB).
    pub async fn resolve(
        pool: &sqlx::PgPool,
        redis: &redis::Client,
        user_id: Uuid,
    ) -> Result<Self, AppError> {
        let permissions = RbacCache::get_or_build(pool, redis, user_id).await?;
        Ok(Self {
            user_id,
            permissions,
        })
    }

    /// Check permission within scope, return 403 if denied.
    pub fn require(&self, permission: &str, scope: &ResourceScope) -> Result<(), AppError> {
        if self.permissions.can(permission, scope) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Missing permission: {}",
                permission
            )))
        }
    }

    /// Convenience: check global permission.
    pub fn require_global(&self, permission: &str) -> Result<(), AppError> {
        self.require(permission, &ResourceScope::global())
    }
}

/// Convenience function for handlers: resolve context and check permission in one call.
pub async fn require_permission(
    pool: &sqlx::PgPool,
    redis: &redis::Client,
    user_id: Uuid,
    permission: &str,
    scope: &ResourceScope,
) -> Result<RbacContext, AppError> {
    let ctx = RbacContext::resolve(pool, redis, user_id).await?;
    ctx.require(permission, scope)?;
    Ok(ctx)
}
