use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::RbacCache;
use crate::models::*;
use crate::repository::RbacRepository;
use olymp_core::auth::AuthContext;
use olymp_core::response::{ApiResponse, Meta, WithStatus};
use olymp_core::types::ListParams;

/// Shared state for RBAC handlers (need both pool and redis for cache invalidation)
#[derive(Clone)]
pub struct RbacState {
    pub pool: PgPool,
    pub redis: redis::Client,
}

// ─── Roles ───

#[utoipa::path(
    get,
    path = "/api/rbac/roles",
    tag = "rbac",
    responses((status = 200, description = "List of roles", body = inline(ApiResponse<Vec<Role>>)))
)]
pub async fn list_roles(auth: AuthContext, State(state): State<RbacState>) -> Response {
    if let Err(e) = auth.require("rbac.role.view") {
        return e.into_response();
    }
    match RbacRepository::list_roles(&state.pool).await {
        Ok(roles) => ApiResponse::success(roles).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/rbac/roles",
    tag = "rbac",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created", body = inline(ApiResponse<Role>)),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_role(
    auth: AuthContext,
    State(state): State<RbacState>,
    Json(req): Json<CreateRoleRequest>,
) -> Response {
    if let Err(e) = auth.require("rbac.role.create") {
        return e.into_response();
    }
    match RbacRepository::create_role(&state.pool, &req.name, req.description.as_deref()).await {
        Ok(role) => WithStatus(StatusCode::CREATED, ApiResponse::success(role)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/rbac/roles/{role_id}",
    tag = "rbac",
    params(("role_id" = Uuid, Path, description = "Role ID")),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated", body = inline(ApiResponse<Role>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_role(
    auth: AuthContext,
    State(state): State<RbacState>,
    Path(role_id): Path<Uuid>,
    Json(req): Json<UpdateRoleRequest>,
) -> Response {
    if let Err(e) = auth.require("rbac.role.update") {
        return e.into_response();
    }
    match RbacRepository::update_role(&state.pool, role_id, &req).await {
        Ok(role) => ApiResponse::success(role).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Permissions ───

#[utoipa::path(
    get,
    path = "/api/rbac/permissions",
    tag = "rbac",
    responses((status = 200, description = "List of all permissions", body = inline(ApiResponse<Vec<Permission>>)))
)]
pub async fn list_permissions(auth: AuthContext, State(state): State<RbacState>) -> Response {
    if let Err(e) = auth.require("rbac.role.view") {
        return e.into_response();
    }
    match RbacRepository::list_permissions(&state.pool).await {
        Ok(perms) => ApiResponse::success(perms).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/rbac/roles/{role_id}/permissions",
    tag = "rbac",
    params(("role_id" = Uuid, Path, description = "Role ID")),
    responses((status = 200, description = "Permissions for role", body = inline(ApiResponse<Vec<Permission>>)))
)]
pub async fn get_role_permissions(
    auth: AuthContext,
    State(state): State<RbacState>,
    Path(role_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("rbac.role.view") {
        return e.into_response();
    }
    match RbacRepository::get_role_permissions(&state.pool, role_id).await {
        Ok(perms) => ApiResponse::success(perms).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/rbac/roles/{role_id}/permissions",
    tag = "rbac",
    params(("role_id" = Uuid, Path, description = "Role ID")),
    request_body = AssignPermissionsRequest,
    responses(
        (status = 200, description = "Permissions assigned"),
        (status = 404, description = "Role not found")
    )
)]
pub async fn assign_role_permissions(
    auth: AuthContext,
    State(state): State<RbacState>,
    Path(role_id): Path<Uuid>,
    Json(req): Json<AssignPermissionsRequest>,
) -> Response {
    if let Err(e) = auth.require("rbac.permission.assign") {
        return e.into_response();
    }
    match RbacRepository::assign_permissions_to_role(&state.pool, role_id, &req.permission_ids)
        .await
    {
        Ok(()) => {
            RbacCache::invalidate_role(&state.pool, &state.redis, role_id).await;
            ApiResponse::success(serde_json::json!({"message": "Permissions assigned"}))
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

// ─── Assignments ───

#[utoipa::path(
    get,
    path = "/api/rbac/assignments",
    tag = "rbac",
    params(ListParams),
    responses((status = 200, description = "Paginated list of role assignments", body = inline(ApiResponse<Vec<UserRoleAssignment>>)))
)]
pub async fn list_assignments(
    auth: AuthContext,
    State(state): State<RbacState>,
    Query(params): Query<ListParams>,
) -> Response {
    if let Err(e) = auth.require("rbac.user.assign") {
        return e.into_response();
    }
    let total = match RbacRepository::count_assignments(&state.pool).await {
        Ok(t) => t,
        Err(e) => return e.into_response(),
    };
    match RbacRepository::list_assignments_paginated(&state.pool, params.limit(), params.offset())
        .await
    {
        Ok(assignments) => ApiResponse::success(assignments)
            .with_meta(Meta::paginated(
                params.page(),
                params.per_page(),
                total as u64,
            ))
            .into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/rbac/assignments",
    tag = "rbac",
    request_body = CreateAssignmentRequest,
    responses(
        (status = 201, description = "Assignment created", body = inline(ApiResponse<UserRoleAssignment>)),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_assignment(
    auth: AuthContext,
    State(state): State<RbacState>,
    Json(req): Json<CreateAssignmentRequest>,
) -> Response {
    if let Err(e) = auth.require("rbac.user.assign") {
        return e.into_response();
    }
    let user_id = req.user_id;
    match RbacRepository::create_assignment(&state.pool, &req, None).await {
        Ok(assignment) => {
            RbacCache::invalidate(&state.redis, user_id).await;
            WithStatus(StatusCode::CREATED, ApiResponse::success(assignment)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/rbac/assignments/{id}",
    tag = "rbac",
    params(("id" = Uuid, Path, description = "Assignment ID")),
    request_body = UpdateAssignmentRequest,
    responses(
        (status = 200, description = "Assignment updated", body = inline(ApiResponse<UserRoleAssignment>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_assignment(
    auth: AuthContext,
    State(state): State<RbacState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateAssignmentRequest>,
) -> Response {
    if let Err(e) = auth.require("rbac.user.assign") {
        return e.into_response();
    }
    match RbacRepository::update_assignment(&state.pool, id, &req).await {
        Ok(assignment) => {
            RbacCache::invalidate(&state.redis, assignment.user_id).await;
            ApiResponse::success(assignment).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/rbac/assignments/{id}",
    tag = "rbac",
    params(("id" = Uuid, Path, description = "Assignment ID")),
    responses(
        (status = 200, description = "Assignment deleted"),
        (status = 404, description = "Not found")
    )
)]
pub async fn delete_assignment(
    auth: AuthContext,
    State(state): State<RbacState>,
    Path(id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("rbac.user.assign") {
        return e.into_response();
    }
    // Get user_id before deletion for cache invalidation
    let assignment = sqlx::query_as::<_, UserRoleAssignment>(
        "SELECT * FROM user_role_assignments WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await;

    match RbacRepository::delete_assignment(&state.pool, id).await {
        Ok(()) => {
            if let Ok(Some(a)) = assignment {
                RbacCache::invalidate(&state.redis, a.user_id).await;
            }
            ApiResponse::success(serde_json::json!({"message": "Assignment deleted"}))
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}
