use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use olymp_core::AppError;

pub struct RbacRepository;

impl RbacRepository {
    // ─── Roles ───

    pub async fn list_roles(pool: &PgPool) -> Result<Vec<Role>, AppError> {
        sqlx::query_as::<_, Role>("SELECT * FROM roles ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn get_role(pool: &PgPool, id: Uuid) -> Result<Option<Role>, AppError> {
        sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn create_role(
        pool: &PgPool,
        name: &str,
        description: Option<&str>,
    ) -> Result<Role, AppError> {
        sqlx::query_as::<_, Role>(
            "INSERT INTO roles (name, description) VALUES ($1, $2) RETURNING *",
        )
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update_role(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateRoleRequest,
    ) -> Result<Role, AppError> {
        let current = Self::get_role(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Role not found".into()))?;

        if current.is_system {
            return Err(AppError::Forbidden("Cannot modify system role".into()));
        }

        let name = req.name.as_deref().unwrap_or(&current.name);
        let description = req.description.as_deref().or(current.description.as_deref());

        sqlx::query_as::<_, Role>(
            "UPDATE roles SET name = $2, description = $3 WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Permissions ───

    pub async fn list_permissions(pool: &PgPool) -> Result<Vec<Permission>, AppError> {
        sqlx::query_as::<_, Permission>("SELECT * FROM permissions ORDER BY resource, action")
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn get_role_permissions(
        pool: &PgPool,
        role_id: Uuid,
    ) -> Result<Vec<Permission>, AppError> {
        sqlx::query_as::<_, Permission>(
            "SELECT p.* FROM permissions p
             JOIN role_permissions rp ON rp.permission_id = p.id
             WHERE rp.role_id = $1
             ORDER BY p.resource, p.action",
        )
        .bind(role_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn assign_permissions_to_role(
        pool: &PgPool,
        role_id: Uuid,
        permission_ids: &[Uuid],
    ) -> Result<(), AppError> {
        // Replace all permissions for role
        let mut tx = pool.begin().await.map_err(AppError::Database)?;

        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .execute(&mut *tx)
            .await
            .map_err(AppError::Database)?;

        for pid in permission_ids {
            sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)")
                .bind(role_id)
                .bind(pid)
                .execute(&mut *tx)
                .await
                .map_err(AppError::Database)?;
        }

        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }

    // ─── Assignments ───

    pub async fn list_assignments(pool: &PgPool) -> Result<Vec<UserRoleAssignment>, AppError> {
        sqlx::query_as::<_, UserRoleAssignment>(
            "SELECT * FROM user_role_assignments ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn list_user_assignments(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<UserRoleAssignment>, AppError> {
        sqlx::query_as::<_, UserRoleAssignment>(
            "SELECT * FROM user_role_assignments WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn create_assignment(
        pool: &PgPool,
        req: &CreateAssignmentRequest,
        assigned_by: Option<Uuid>,
    ) -> Result<UserRoleAssignment, AppError> {
        sqlx::query_as::<_, UserRoleAssignment>(
            "INSERT INTO user_role_assignments (user_id, role_id, event_id, stage_id, province_id, district_id, expires_at, assigned_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
        )
        .bind(req.user_id)
        .bind(req.role_id)
        .bind(req.event_id)
        .bind(req.stage_id)
        .bind(req.province_id)
        .bind(req.district_id)
        .bind(req.expires_at)
        .bind(assigned_by)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update_assignment(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateAssignmentRequest,
    ) -> Result<UserRoleAssignment, AppError> {
        let current = sqlx::query_as::<_, UserRoleAssignment>(
            "SELECT * FROM user_role_assignments WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Assignment not found".into()))?;

        let is_active = req.is_active.unwrap_or(current.is_active);
        let expires_at = req.expires_at.or(current.expires_at);

        sqlx::query_as::<_, UserRoleAssignment>(
            "UPDATE user_role_assignments SET is_active = $2, expires_at = $3, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(is_active)
        .bind(expires_at)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn delete_assignment(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM user_role_assignments WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Assignment not found".into()));
        }
        Ok(())
    }

    // ─── Effective Permissions (for caching) ───

    pub async fn build_effective_permissions(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<EffectivePermissions, AppError> {
        // Get active, non-expired assignments
        let assignments = sqlx::query_as::<_, UserRoleAssignment>(
            "SELECT * FROM user_role_assignments
             WHERE user_id = $1 AND is_active = true
             AND (expires_at IS NULL OR expires_at > now())",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        let mut effective = Vec::new();

        for assignment in &assignments {
            // Get role name
            let role = sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = $1")
                .bind(assignment.role_id)
                .fetch_one(pool)
                .await
                .map_err(AppError::Database)?;

            // Get role permissions
            let perms = Self::get_role_permissions(pool, assignment.role_id).await?;
            let mut perm_codes: std::collections::HashSet<String> =
                perms.iter().map(|p| p.code.clone()).collect();

            // Apply overrides
            let overrides = sqlx::query_as::<_, AssignmentPermissionOverride>(
                "SELECT * FROM assignment_permission_overrides WHERE assignment_id = $1",
            )
            .bind(assignment.id)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

            for ov in overrides {
                if let Some(perm) = sqlx::query_as::<_, Permission>(
                    "SELECT * FROM permissions WHERE id = $1",
                )
                .bind(ov.permission_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::Database)?
                {
                    if ov.granted {
                        perm_codes.insert(perm.code);
                    } else {
                        perm_codes.remove(&perm.code);
                    }
                }
            }

            effective.push(EffectiveAssignment {
                role: role.name,
                permissions: perm_codes,
                event_id: assignment.event_id,
                stage_id: assignment.stage_id,
                province_id: assignment.province_id,
                district_id: assignment.district_id,
            });
        }

        Ok(EffectivePermissions {
            user_id,
            assignments: effective,
        })
    }
}
