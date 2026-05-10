use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use std::collections::HashSet;
use uuid::Uuid;

use crate::AppError;

/// Auth context injected by middleware into every authenticated request.
/// Contains user_id, resolved roles, and flat permission set.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub roles: Vec<String>,
    pub permissions: HashSet<String>,
    /// Whether user has any global (unscoped) role assignment
    pub is_global: bool,
    /// Event IDs user is scoped to (empty if global)
    pub scoped_events: Vec<Uuid>,
}

impl AuthContext {
    /// Check if user has permission (global check only)
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }

    /// Check if user has specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user is admin-level (superadmin or admin)
    pub fn is_admin(&self) -> bool {
        self.has_role("superadmin") || self.has_role("admin")
    }

    /// Check if user is staff (admin, superadmin, or panitia)
    pub fn is_staff(&self) -> bool {
        self.is_admin() || self.has_role("panitia")
    }

    /// Require permission or return 403
    pub fn require(&self, permission: &str) -> Result<(), AppError> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "Missing permission: {}",
                permission
            )))
        }
    }

    /// Require staff role or return 403
    pub fn require_staff(&self) -> Result<(), AppError> {
        if self.is_staff() {
            Ok(())
        } else {
            Err(AppError::Forbidden("Staff access required".into()))
        }
    }
}

/// Axum extractor: pulls AuthContext from request extensions (set by middleware).
impl<S: Send + Sync> FromRequestParts<S> for AuthContext {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthContext>()
            .cloned()
            .ok_or_else(|| {
                (StatusCode::UNAUTHORIZED, "Authentication required").into_response()
            })
    }
}
