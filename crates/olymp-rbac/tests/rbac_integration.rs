mod common;

use common::TestDb;
use olymp_rbac::models::*;
use olymp_rbac::repository::RbacRepository;

// ─── Seed Verification: default roles + permissions exist ───

#[tokio::test]
async fn seed_creates_four_system_roles() {
    let db = TestDb::new().await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();

    let system_roles: Vec<&str> = roles
        .iter()
        .filter(|r| r.is_system)
        .map(|r| r.name.as_str())
        .collect();

    assert!(system_roles.contains(&"superadmin"), "missing superadmin");
    assert!(system_roles.contains(&"admin"), "missing admin");
    assert!(system_roles.contains(&"panitia"), "missing panitia");
    assert!(system_roles.contains(&"peserta"), "missing peserta");
    assert_eq!(system_roles.len(), 4, "expected exactly 4 system roles");
}

#[tokio::test]
async fn seed_creates_31_permissions() {
    let db = TestDb::new().await;
    let perms = RbacRepository::list_permissions(&db.pool).await.unwrap();

    assert_eq!(perms.len(), 31, "expected 31 seeded permissions, got {}", perms.len());

    // Spot-check key permissions exist
    let codes: Vec<&str> = perms.iter().map(|p| p.code.as_str()).collect();
    assert!(codes.contains(&"exam.view"));
    assert!(codes.contains(&"rbac.user.assign"));
    assert!(codes.contains(&"certificate.generate"));
    assert!(codes.contains(&"ranking.promote"));
    assert!(codes.contains(&"monitoring.flag"));
}

#[tokio::test]
async fn superadmin_role_has_all_31_permissions() {
    let db = TestDb::new().await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let superadmin = roles.iter().find(|r| r.name == "superadmin").unwrap();

    let perms = RbacRepository::get_role_permissions(&db.pool, superadmin.id)
        .await
        .unwrap();

    assert_eq!(perms.len(), 31, "superadmin should have all 31 permissions, got {}", perms.len());
}

#[tokio::test]
async fn admin_role_has_no_rbac_permissions() {
    let db = TestDb::new().await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let admin = roles.iter().find(|r| r.name == "admin").unwrap();

    let perms = RbacRepository::get_role_permissions(&db.pool, admin.id)
        .await
        .unwrap();

    let rbac_perms: Vec<&str> = perms
        .iter()
        .filter(|p| p.resource == "rbac")
        .map(|p| p.code.as_str())
        .collect();

    assert!(rbac_perms.is_empty(), "admin should have NO rbac perms, found: {:?}", rbac_perms);

    // But has all non-rbac perms
    let non_rbac_count = perms.iter().filter(|p| p.resource != "rbac").count();
    assert_eq!(non_rbac_count, 25, "admin should have 25 non-rbac permissions, got {}", non_rbac_count);
}

#[tokio::test]
async fn panitia_role_has_exactly_10_permissions() {
    let db = TestDb::new().await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let panitia = roles.iter().find(|r| r.name == "panitia").unwrap();

    let perms = RbacRepository::get_role_permissions(&db.pool, panitia.id)
        .await
        .unwrap();

    let codes: Vec<&str> = perms.iter().map(|p| p.code.as_str()).collect();
    let expected = vec![
        "participant.view", "participant.verify", "participant.approve", "participant.reject",
        "exam.view", "exam.monitor",
        "monitoring.view", "monitoring.flag",
        "ranking.view", "region.view",
    ];

    assert_eq!(perms.len(), expected.len(), "panitia should have {} perms, got {}", expected.len(), perms.len());
    for exp in &expected {
        assert!(codes.contains(exp), "panitia missing: {}", exp);
    }
}

#[tokio::test]
async fn peserta_role_has_exactly_5_permissions() {
    let db = TestDb::new().await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let peserta = roles.iter().find(|r| r.name == "peserta").unwrap();

    let perms = RbacRepository::get_role_permissions(&db.pool, peserta.id)
        .await
        .unwrap();

    let codes: Vec<&str> = perms.iter().map(|p| p.code.as_str()).collect();
    let expected = vec![
        "participant.view", "exam.view", "ranking.view", "certificate.view", "region.view",
    ];

    assert_eq!(perms.len(), expected.len(), "peserta should have {} perms, got {}", expected.len(), perms.len());
    for exp in &expected {
        assert!(codes.contains(exp), "peserta missing: {}", exp);
    }
}

// ─── Cannot modify system roles ───

#[tokio::test]
async fn cannot_modify_system_role() {
    let db = TestDb::new().await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let superadmin = roles.iter().find(|r| r.name == "superadmin").unwrap();

    let result = RbacRepository::update_role(
        &db.pool,
        superadmin.id,
        &UpdateRoleRequest {
            name: Some("hacked".into()),
            description: None,
        },
    )
    .await;

    assert!(result.is_err(), "should not be able to modify system role");
}

// ─── Custom role CRUD ───

#[tokio::test]
async fn create_and_modify_custom_role() {
    let db = TestDb::new().await;

    let role = RbacRepository::create_role(&db.pool, "reviewer", Some("Custom reviewer"))
        .await
        .unwrap();

    assert_eq!(role.name, "reviewer");
    assert!(!role.is_system);

    let updated = RbacRepository::update_role(
        &db.pool,
        role.id,
        &UpdateRoleRequest {
            name: Some("senior_reviewer".into()),
            description: Some("Updated description".into()),
        },
    )
    .await
    .unwrap();

    assert_eq!(updated.name, "senior_reviewer");
    assert_eq!(updated.description.as_deref(), Some("Updated description"));

    // Cleanup
    sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(role.id)
        .execute(&db.pool)
        .await
        .ok();
}

// ─── build_effective_permissions for each role ───

#[tokio::test]
async fn build_effective_perms_superadmin_global() {
    let db = TestDb::new().await;
    let user_id = db.create_test_user("superadmin@test.com").await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let superadmin = roles.iter().find(|r| r.name == "superadmin").unwrap();

    // Create global assignment
    RbacRepository::create_assignment(
        &db.pool,
        &CreateAssignmentRequest {
            user_id,
            role_id: superadmin.id,
            event_id: None,
            stage_id: None,
            province_id: None,
            district_id: None,
            expires_at: None,
        },
        None,
    )
    .await
    .unwrap();

    let effective = RbacRepository::build_effective_permissions(&db.pool, user_id)
        .await
        .unwrap();

    assert_eq!(effective.assignments.len(), 1);
    assert_eq!(effective.assignments[0].role, "superadmin");
    assert_eq!(effective.assignments[0].permissions.len(), 31);
    assert!(effective.is_global());

    // Can do everything
    assert!(effective.can("rbac.user.assign", &ResourceScope::global()));
    assert!(effective.can("exam.create", &ResourceScope::event(uuid::Uuid::from_u128(42))));
    assert!(effective.can("certificate.generate", &ResourceScope::global()));

    // Cleanup
    cleanup_user(&db.pool, user_id).await;
}

#[tokio::test]
async fn build_effective_perms_peserta_no_write() {
    let db = TestDb::new().await;
    let user_id = db.create_test_user("peserta@test.com").await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let peserta = roles.iter().find(|r| r.name == "peserta").unwrap();

    RbacRepository::create_assignment(
        &db.pool,
        &CreateAssignmentRequest {
            user_id,
            role_id: peserta.id,
            event_id: None,
            stage_id: None,
            province_id: None,
            district_id: None,
            expires_at: None,
        },
        None,
    )
    .await
    .unwrap();

    let effective = RbacRepository::build_effective_permissions(&db.pool, user_id)
        .await
        .unwrap();

    assert_eq!(effective.assignments[0].permissions.len(), 5);
    assert!(effective.can("exam.view", &ResourceScope::global()));
    assert!(!effective.can("exam.create", &ResourceScope::global()));
    assert!(!effective.can("rbac.role.create", &ResourceScope::global()));

    cleanup_user(&db.pool, user_id).await;
}

#[tokio::test]
async fn build_effective_perms_panitia_scoped() {
    let db = TestDb::new().await;
    let user_id = db.create_test_user("panitia@test.com").await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let panitia = roles.iter().find(|r| r.name == "panitia").unwrap();

    // Create an event for scoping
    let event_id = uuid::Uuid::now_v7();
    let slug = format!("test-event-{}", &event_id.to_string()[..8]);
    sqlx::query(
        "INSERT INTO events (id, name, slug, academic_year, status) VALUES ($1, 'Test Event', $2, '2025/2026', 'draft')",
    )
    .bind(event_id)
    .bind(&slug)
    .execute(&db.pool)
    .await
    .unwrap();

    // Assign panitia scoped to event
    RbacRepository::create_assignment(
        &db.pool,
        &CreateAssignmentRequest {
            user_id,
            role_id: panitia.id,
            event_id: Some(event_id),
            stage_id: None,
            province_id: None,
            district_id: None,
            expires_at: None,
        },
        None,
    )
    .await
    .unwrap();

    let effective = RbacRepository::build_effective_permissions(&db.pool, user_id)
        .await
        .unwrap();

    assert_eq!(effective.assignments.len(), 1);
    assert_eq!(effective.assignments[0].event_id, Some(event_id));
    assert!(!effective.is_global());

    // Can view participants in assigned event
    assert!(effective.can("participant.view", &ResourceScope::event(event_id)));
    // Cannot view in other event
    assert!(!effective.can("participant.view", &ResourceScope::event(uuid::Uuid::from_u128(999))));
    // Cannot create exams
    assert!(!effective.can("exam.create", &ResourceScope::event(event_id)));

    // Cleanup
    sqlx::query("DELETE FROM user_role_assignments WHERE user_id = $1")
        .bind(user_id)
        .execute(&db.pool)
        .await
        .ok();
    sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(event_id)
        .execute(&db.pool)
        .await
        .ok();
    cleanup_user(&db.pool, user_id).await;
}

// ─── Assignment lifecycle ───

#[tokio::test]
async fn assignment_lifecycle_create_deactivate_delete() {
    let db = TestDb::new().await;
    let user_id = db.create_test_user("lifecycle@test.com").await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let peserta = roles.iter().find(|r| r.name == "peserta").unwrap();

    // Create
    let assignment = RbacRepository::create_assignment(
        &db.pool,
        &CreateAssignmentRequest {
            user_id,
            role_id: peserta.id,
            event_id: None,
            stage_id: None,
            province_id: None,
            district_id: None,
            expires_at: None,
        },
        None,
    )
    .await
    .unwrap();
    assert!(assignment.is_active);

    // User should have permissions
    let effective = RbacRepository::build_effective_permissions(&db.pool, user_id)
        .await
        .unwrap();
    assert_eq!(effective.assignments.len(), 1);

    // Deactivate
    let updated = RbacRepository::update_assignment(
        &db.pool,
        assignment.id,
        &UpdateAssignmentRequest {
            is_active: Some(false),
            expires_at: None,
        },
    )
    .await
    .unwrap();
    assert!(!updated.is_active);

    // User should have NO effective permissions (inactive filtered out)
    let effective = RbacRepository::build_effective_permissions(&db.pool, user_id)
        .await
        .unwrap();
    assert_eq!(effective.assignments.len(), 0);

    // Reactivate
    RbacRepository::update_assignment(
        &db.pool,
        assignment.id,
        &UpdateAssignmentRequest {
            is_active: Some(true),
            expires_at: None,
        },
    )
    .await
    .unwrap();

    let effective = RbacRepository::build_effective_permissions(&db.pool, user_id)
        .await
        .unwrap();
    assert_eq!(effective.assignments.len(), 1);

    // Delete
    RbacRepository::delete_assignment(&db.pool, assignment.id)
        .await
        .unwrap();

    let effective = RbacRepository::build_effective_permissions(&db.pool, user_id)
        .await
        .unwrap();
    assert_eq!(effective.assignments.len(), 0);

    cleanup_user(&db.pool, user_id).await;
}

#[tokio::test]
async fn expired_assignment_excluded_from_effective() {
    let db = TestDb::new().await;
    let user_id = db.create_test_user("expired@test.com").await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let peserta = roles.iter().find(|r| r.name == "peserta").unwrap();

    // Create assignment that expired 1 hour ago
    let expired_at = chrono::Utc::now() - chrono::Duration::hours(1);

    RbacRepository::create_assignment(
        &db.pool,
        &CreateAssignmentRequest {
            user_id,
            role_id: peserta.id,
            event_id: None,
            stage_id: None,
            province_id: None,
            district_id: None,
            expires_at: Some(expired_at),
        },
        None,
    )
    .await
    .unwrap();

    // Should be filtered out (expired)
    let effective = RbacRepository::build_effective_permissions(&db.pool, user_id)
        .await
        .unwrap();
    assert_eq!(effective.assignments.len(), 0, "expired assignment should not appear");

    cleanup_user(&db.pool, user_id).await;
}

#[tokio::test]
async fn multiple_role_assignments_combine() {
    let db = TestDb::new().await;
    let user_id = db.create_test_user("multirole@test.com").await;
    let roles = RbacRepository::list_roles(&db.pool).await.unwrap();
    let peserta = roles.iter().find(|r| r.name == "peserta").unwrap();
    let admin = roles.iter().find(|r| r.name == "admin").unwrap();

    // Peserta globally
    RbacRepository::create_assignment(
        &db.pool,
        &CreateAssignmentRequest {
            user_id,
            role_id: peserta.id,
            event_id: None,
            stage_id: None,
            province_id: None,
            district_id: None,
            expires_at: None,
        },
        None,
    )
    .await
    .unwrap();

    // Admin for specific event
    let event_id = uuid::Uuid::now_v7();
    let slug = format!("multi-event-{}", &event_id.to_string()[..8]);
    sqlx::query("INSERT INTO events (id, name, slug, academic_year, status) VALUES ($1, 'Multi Event', $2, '2025/2026', 'draft')")
        .bind(event_id)
        .bind(&slug)
        .execute(&db.pool)
        .await
        .unwrap();

    RbacRepository::create_assignment(
        &db.pool,
        &CreateAssignmentRequest {
            user_id,
            role_id: admin.id,
            event_id: Some(event_id),
            stage_id: None,
            province_id: None,
            district_id: None,
            expires_at: None,
        },
        None,
    )
    .await
    .unwrap();

    let effective = RbacRepository::build_effective_permissions(&db.pool, user_id)
        .await
        .unwrap();

    assert_eq!(effective.assignments.len(), 2);

    // Peserta perms work globally
    assert!(effective.can("exam.view", &ResourceScope::global()));
    assert!(effective.can("certificate.view", &ResourceScope::global()));

    // Admin perms work for assigned event only
    assert!(effective.can("exam.create", &ResourceScope::event(event_id)));
    assert!(!effective.can("exam.create", &ResourceScope::event(uuid::Uuid::from_u128(999))));

    // Admin still has no RBAC perms
    assert!(!effective.can("rbac.role.create", &ResourceScope::event(event_id)));

    // Cleanup
    sqlx::query("DELETE FROM user_role_assignments WHERE user_id = $1")
        .bind(user_id)
        .execute(&db.pool)
        .await
        .ok();
    sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(event_id)
        .execute(&db.pool)
        .await
        .ok();
    cleanup_user(&db.pool, user_id).await;
}

// ─── Delete nonexistent assignment ───

#[tokio::test]
async fn delete_nonexistent_assignment_fails() {
    let db = TestDb::new().await;
    let result = RbacRepository::delete_assignment(&db.pool, uuid::Uuid::from_u128(999999)).await;
    assert!(result.is_err());
}

// ─── Helpers ───

async fn cleanup_user(pool: &sqlx::PgPool, user_id: uuid::Uuid) {
    sqlx::query("DELETE FROM user_role_assignments WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM auth.users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
}
