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

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Helpers ───

    fn uuid(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn make_perms(assignments: Vec<EffectiveAssignment>) -> EffectivePermissions {
        EffectivePermissions {
            user_id: uuid(1),
            assignments,
        }
    }

    fn global_assignment(role: &str, perms: &[&str]) -> EffectiveAssignment {
        EffectiveAssignment {
            role: role.into(),
            permissions: perms.iter().map(|s| s.to_string()).collect(),
            event_id: None,
            stage_id: None,
            province_id: None,
            district_id: None,
        }
    }

    fn scoped_assignment(
        role: &str,
        perms: &[&str],
        event_id: Option<Uuid>,
        stage_id: Option<Uuid>,
        province_id: Option<Uuid>,
        district_id: Option<Uuid>,
    ) -> EffectiveAssignment {
        EffectiveAssignment {
            role: role.into(),
            permissions: perms.iter().map(|s| s.to_string()).collect(),
            event_id,
            stage_id,
            province_id,
            district_id,
        }
    }

    // ─── 1. EffectivePermissions.can() with scopes ───

    #[test]
    fn global_assignment_grants_any_scope() {
        let perms = make_perms(vec![global_assignment("superadmin", &["exam.view", "exam.create"])]);
        // Global scope
        assert!(perms.can("exam.view", &ResourceScope::global()));
        // Event scope
        assert!(perms.can("exam.view", &ResourceScope::event(uuid(100))));
        // Stage scope
        assert!(perms.can("exam.create", &ResourceScope::stage(uuid(100), uuid(200))));
    }

    #[test]
    fn scoped_assignment_matches_exact_scope() {
        let event_id = uuid(100);
        let perms = make_perms(vec![scoped_assignment(
            "admin",
            &["exam.view", "participant.view"],
            Some(event_id),
            None, None, None,
        )]);

        // Same event → allowed
        assert!(perms.can("exam.view", &ResourceScope::event(event_id)));
        // Different event → denied
        assert!(!perms.can("exam.view", &ResourceScope::event(uuid(999))));
    }

    #[test]
    fn scoped_assignment_denies_global_query() {
        let perms = make_perms(vec![scoped_assignment(
            "admin",
            &["exam.view"],
            Some(uuid(100)),
            None, None, None,
        )]);

        // Asking globally — scoped assignment has event_id set,
        // ResourceScope::global() has event_id=None, so scope_matches(Some(100), None) → false
        assert!(!perms.can("exam.view", &ResourceScope::global()));
    }

    #[test]
    fn missing_permission_denied() {
        let perms = make_perms(vec![global_assignment("peserta", &["exam.view"])]);
        assert!(!perms.can("exam.create", &ResourceScope::global()));
    }

    #[test]
    fn empty_assignments_deny_all() {
        let perms = make_perms(vec![]);
        assert!(!perms.can("exam.view", &ResourceScope::global()));
    }

    // ─── 2. Superadmin/Admin/Panitia/Peserta scenarios ───

    /// Superadmin: all 31 permissions, global scope
    #[test]
    fn superadmin_has_all_permissions_globally() {
        let all_perms = vec![
            "olympiad.master.create", "olympiad.master.update", "olympiad.master.delete",
            "olympiad.stage.manage", "region.view", "region.manage",
            "participant.view", "participant.create", "participant.update",
            "participant.import", "participant.verify", "participant.approve", "participant.reject",
            "exam.view", "exam.create", "exam.update", "exam.assign", "exam.monitor",
            "ranking.view", "ranking.approve", "ranking.promote",
            "monitoring.view", "monitoring.flag",
            "rbac.role.view", "rbac.role.create", "rbac.role.update",
            "rbac.permission.assign", "rbac.user.assign", "rbac.audit.view",
            "certificate.generate", "certificate.view",
        ];

        let perms = make_perms(vec![global_assignment("superadmin", &all_perms)]);

        for p in &all_perms {
            assert!(perms.can(p, &ResourceScope::global()), "superadmin missing: {}", p);
            assert!(perms.can(p, &ResourceScope::event(uuid(42))), "superadmin missing scoped: {}", p);
        }
    }

    /// Admin: everything except RBAC, global scope
    #[test]
    fn admin_has_no_rbac_permissions() {
        let admin_perms = vec![
            "olympiad.master.create", "olympiad.master.update", "olympiad.master.delete",
            "olympiad.stage.manage", "region.view", "region.manage",
            "participant.view", "participant.create", "participant.update",
            "participant.import", "participant.verify", "participant.approve", "participant.reject",
            "exam.view", "exam.create", "exam.update", "exam.assign", "exam.monitor",
            "ranking.view", "ranking.approve", "ranking.promote",
            "monitoring.view", "monitoring.flag",
            "certificate.generate", "certificate.view",
        ];

        let perms = make_perms(vec![global_assignment("admin", &admin_perms)]);

        // Has non-RBAC perms
        assert!(perms.can("exam.create", &ResourceScope::global()));
        assert!(perms.can("participant.approve", &ResourceScope::global()));
        assert!(perms.can("ranking.promote", &ResourceScope::global()));

        // Does NOT have RBAC perms
        assert!(!perms.can("rbac.role.create", &ResourceScope::global()));
        assert!(!perms.can("rbac.user.assign", &ResourceScope::global()));
        assert!(!perms.can("rbac.audit.view", &ResourceScope::global()));
    }

    /// Panitia: scoped to event, limited operational perms
    #[test]
    fn panitia_scoped_to_event() {
        let event_id = uuid(500);
        let panitia_perms = vec![
            "participant.view", "participant.verify", "participant.approve", "participant.reject",
            "exam.view", "exam.monitor",
            "monitoring.view", "monitoring.flag",
            "ranking.view", "region.view",
        ];

        let perms = make_perms(vec![scoped_assignment(
            "panitia",
            &panitia_perms,
            Some(event_id),
            None, None, None,
        )]);

        // Can view participants in assigned event
        assert!(perms.can("participant.view", &ResourceScope::event(event_id)));
        // Can verify in assigned event
        assert!(perms.can("participant.verify", &ResourceScope::event(event_id)));
        // Cannot view in different event
        assert!(!perms.can("participant.view", &ResourceScope::event(uuid(999))));
        // Cannot create exams (not in perm set)
        assert!(!perms.can("exam.create", &ResourceScope::event(event_id)));
        // Cannot manage RBAC
        assert!(!perms.can("rbac.role.create", &ResourceScope::global()));
    }

    /// Peserta: minimal self-service
    #[test]
    fn peserta_minimal_permissions() {
        let peserta_perms = vec![
            "participant.view", "exam.view", "ranking.view", "certificate.view", "region.view",
        ];

        let perms = make_perms(vec![global_assignment("peserta", &peserta_perms)]);

        // Can view exams, rankings, certificates
        assert!(perms.can("exam.view", &ResourceScope::global()));
        assert!(perms.can("ranking.view", &ResourceScope::global()));
        assert!(perms.can("certificate.view", &ResourceScope::global()));

        // Cannot create/update anything
        assert!(!perms.can("exam.create", &ResourceScope::global()));
        assert!(!perms.can("participant.create", &ResourceScope::global()));
        assert!(!perms.can("participant.approve", &ResourceScope::global()));
        assert!(!perms.can("ranking.approve", &ResourceScope::global()));
        assert!(!perms.can("certificate.generate", &ResourceScope::global()));
    }

    // ─── 3. scope_matches edge cases ───

    #[test]
    fn scope_matches_both_none() {
        assert!(scope_matches(None, None));
    }

    #[test]
    fn scope_matches_assignment_none_resource_some() {
        // Assignment unscoped → matches any resource
        assert!(scope_matches(None, Some(uuid(1))));
    }

    #[test]
    fn scope_matches_assignment_some_resource_none() {
        // Assignment scoped, resource global → no match
        assert!(!scope_matches(Some(uuid(1)), None));
    }

    #[test]
    fn scope_matches_same_id() {
        assert!(scope_matches(Some(uuid(1)), Some(uuid(1))));
    }

    #[test]
    fn scope_matches_different_id() {
        assert!(!scope_matches(Some(uuid(1)), Some(uuid(2))));
    }

    #[test]
    fn multi_dimension_scope_all_match() {
        let event_id = uuid(10);
        let province_id = uuid(20);

        let perms = make_perms(vec![scoped_assignment(
            "panitia",
            &["participant.view"],
            Some(event_id),
            None,
            Some(province_id),
            None,
        )]);

        let scope = ResourceScope {
            event_id: Some(event_id),
            stage_id: None,
            province_id: Some(province_id),
            district_id: None,
        };

        assert!(perms.can("participant.view", &scope));
    }

    #[test]
    fn multi_dimension_scope_partial_mismatch() {
        let perms = make_perms(vec![scoped_assignment(
            "panitia",
            &["participant.view"],
            Some(uuid(10)),
            None,
            Some(uuid(20)),
            None,
        )]);

        // Event matches, province doesn't
        let scope = ResourceScope {
            event_id: Some(uuid(10)),
            stage_id: None,
            province_id: Some(uuid(99)),
            district_id: None,
        };

        assert!(!perms.can("participant.view", &scope));
    }

    #[test]
    fn stage_scoped_assignment() {
        let event_id = uuid(10);
        let stage_id = uuid(20);

        let perms = make_perms(vec![scoped_assignment(
            "panitia",
            &["exam.monitor"],
            Some(event_id),
            Some(stage_id),
            None, None,
        )]);

        // Right event + stage → allowed
        assert!(perms.can("exam.monitor", &ResourceScope::stage(event_id, stage_id)));
        // Right event, wrong stage → denied
        assert!(!perms.can("exam.monitor", &ResourceScope::stage(event_id, uuid(99))));
        // Event only → denied (assignment has stage_id set)
        assert!(!perms.can("exam.monitor", &ResourceScope::event(event_id)));
    }

    // ─── 4. Multiple assignments combine ───

    #[test]
    fn multiple_assignments_union_permissions() {
        let perms = make_perms(vec![
            scoped_assignment("panitia", &["exam.view"], Some(uuid(10)), None, None, None),
            scoped_assignment("panitia", &["exam.view"], Some(uuid(20)), None, None, None),
        ]);

        // Both events accessible
        assert!(perms.can("exam.view", &ResourceScope::event(uuid(10))));
        assert!(perms.can("exam.view", &ResourceScope::event(uuid(20))));
        // Other event still denied
        assert!(!perms.can("exam.view", &ResourceScope::event(uuid(30))));
    }

    #[test]
    fn mixed_global_and_scoped_assignments() {
        let perms = make_perms(vec![
            global_assignment("peserta", &["exam.view", "certificate.view"]),
            scoped_assignment("panitia", &["participant.verify"], Some(uuid(10)), None, None, None),
        ]);

        // Peserta global perms work everywhere
        assert!(perms.can("exam.view", &ResourceScope::global()));
        assert!(perms.can("exam.view", &ResourceScope::event(uuid(999))));
        // Panitia scoped perm only in right event
        assert!(perms.can("participant.verify", &ResourceScope::event(uuid(10))));
        assert!(!perms.can("participant.verify", &ResourceScope::event(uuid(20))));
    }

    // ─── 5. Utility methods ───

    #[test]
    fn accessible_events_returns_unique() {
        let perms = make_perms(vec![
            scoped_assignment("panitia", &["exam.view"], Some(uuid(10)), None, None, None),
            scoped_assignment("panitia", &["exam.view"], Some(uuid(20)), None, None, None),
            scoped_assignment("panitia", &["participant.view"], Some(uuid(10)), None, None, None),
        ]);

        let events = perms.accessible_events();
        assert_eq!(events.len(), 2);
        assert!(events.contains(&uuid(10)));
        assert!(events.contains(&uuid(20)));
    }

    #[test]
    fn is_global_true_for_unscoped() {
        let perms = make_perms(vec![global_assignment("superadmin", &["exam.view"])]);
        assert!(perms.is_global());
    }

    #[test]
    fn is_global_false_for_scoped_only() {
        let perms = make_perms(vec![scoped_assignment(
            "panitia", &["exam.view"], Some(uuid(10)), None, None, None,
        )]);
        assert!(!perms.is_global());
    }

    // ─── 6. ResourceScope constructors ───

    #[test]
    fn resource_scope_constructors() {
        let g = ResourceScope::global();
        assert!(g.event_id.is_none());
        assert!(g.stage_id.is_none());

        let e = ResourceScope::event(uuid(1));
        assert_eq!(e.event_id, Some(uuid(1)));
        assert!(e.stage_id.is_none());

        let s = ResourceScope::stage(uuid(1), uuid(2));
        assert_eq!(s.event_id, Some(uuid(1)));
        assert_eq!(s.stage_id, Some(uuid(2)));
    }
}
