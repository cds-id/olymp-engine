use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashSet;
use uuid::Uuid;

// ─── DB Models ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Permission {
    pub id: Uuid,
    pub code: String,
    pub resource: String,
    pub action: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UserRoleAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub event_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub province_id: Option<Uuid>,
    pub district_id: Option<Uuid>,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub assigned_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AssignmentPermissionOverride {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub permission_id: Uuid,
    pub granted: bool,
}

// ─── Request DTOs ───

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AssignPermissionsRequest {
    pub permission_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateAssignmentRequest {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub event_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub province_id: Option<Uuid>,
    pub district_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateAssignmentRequest {
    pub is_active: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
}

// ─── Effective Permission Context (cached in Redis) ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveAssignment {
    pub role: String,
    pub permissions: HashSet<String>,
    pub event_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub province_id: Option<Uuid>,
    pub district_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivePermissions {
    pub user_id: Uuid,
    pub assignments: Vec<EffectiveAssignment>,
}

/// Scope for checking whether user can access a specific resource
#[derive(Debug, Default)]
pub struct ResourceScope {
    pub event_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub province_id: Option<Uuid>,
    pub district_id: Option<Uuid>,
}

impl ResourceScope {
    pub fn event(event_id: Uuid) -> Self {
        Self { event_id: Some(event_id), ..Default::default() }
    }

    pub fn stage(event_id: Uuid, stage_id: Uuid) -> Self {
        Self { event_id: Some(event_id), stage_id: Some(stage_id), ..Default::default() }
    }

    pub fn global() -> Self {
        Self::default()
    }
}

impl EffectivePermissions {
    /// Check if user has permission within given scope.
    /// Global assignments (no scope) match everything.
    /// Scoped assignments match if all non-None scope fields match.
    pub fn can(&self, permission: &str, scope: &ResourceScope) -> bool {
        self.assignments.iter().any(|a| {
            if !a.permissions.contains(permission) {
                return false;
            }
            // Global assignment matches any scope
            if a.event_id.is_none()
                && a.stage_id.is_none()
                && a.province_id.is_none()
                && a.district_id.is_none()
            {
                return true;
            }
            // Scoped: check each non-None field
            scope_matches(a.event_id, scope.event_id)
                && scope_matches(a.stage_id, scope.stage_id)
                && scope_matches(a.province_id, scope.province_id)
                && scope_matches(a.district_id, scope.district_id)
        })
    }

    /// All event IDs this user has any access to
    pub fn accessible_events(&self) -> Vec<Uuid> {
        self.assignments
            .iter()
            .filter_map(|a| a.event_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Whether user has any global (unscoped) assignment
    pub fn is_global(&self) -> bool {
        self.assignments.iter().any(|a| {
            a.event_id.is_none()
                && a.stage_id.is_none()
                && a.province_id.is_none()
                && a.district_id.is_none()
        })
    }
}

/// If assignment scope field is None → doesn't constrain (matches any).
/// If set → must match resource scope (or resource scope is None → no match).
fn scope_matches(assignment_scope: Option<Uuid>, resource_scope: Option<Uuid>) -> bool {
    match assignment_scope {
        None => true, // assignment not scoped on this dimension
        Some(a) => resource_scope.map_or(false, |r| a == r),
    }
}
