use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use olymp_core::auth::AuthContext;
use std::collections::HashSet;
use uuid::Uuid;

/// Shared state for auth middleware resolution
#[derive(Clone)]
pub struct AuthState {
    pub db: sqlx::PgPool,
    pub redis: redis::Client,
    pub jwt_secret: String,
    pub jwt_access_ttl_secs: u64,
}

/// Routes that skip authentication entirely
const PUBLIC_PATHS: &[&str] = &[
    "/health",
    "/api/auth/register",
    "/api/auth/login",
    "/api/auth/magic-link",
    "/api/auth/callback",
];

const PUBLIC_PREFIXES: &[&str] = &["/api-docs/", "/swagger-ui/"];

pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AuthState>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();

    // Skip public routes
    if PUBLIC_PATHS.contains(&path.as_str())
        || PUBLIC_PREFIXES.iter().any(|p| path.starts_with(p))
    {
        return next.run(request).await;
    }

    // Extract Bearer token
    let token = match request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(t) => t.to_string(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "data": null,
                    "error": {"code": "UNAUTHORIZED", "message": "Missing or invalid Authorization header"},
                    "meta": null
                })),
            )
                .into_response();
        }
    };

    // Verify JWT
    let jwt_service =
        olymp_auth::JwtService::new(state.jwt_secret.clone(), state.jwt_access_ttl_secs);
    let claims = match jwt_service.verify(&token) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "data": null,
                    "error": {"code": "UNAUTHORIZED", "message": "Invalid or expired token"},
                    "meta": null
                })),
            )
                .into_response();
        }
    };

    // Check Redis denylist (revoked tokens)
    if let Ok(mut conn) = state.redis.get_multiplexed_async_connection().await {
        let key = format!("jwt:deny:{}", claims.jti);
        let revoked: i32 = redis::AsyncCommands::exists(&mut conn, &key)
            .await
            .unwrap_or(0);
        if revoked > 0 {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "data": null,
                    "error": {"code": "UNAUTHORIZED", "message": "Token has been revoked"},
                    "meta": null
                })),
            )
                .into_response();
        }
    }

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "data": null,
                    "error": {"code": "UNAUTHORIZED", "message": "Invalid user ID in token"},
                    "meta": null
                })),
            )
                .into_response();
        }
    };

    // Resolve RBAC permissions (Redis-cached)
    let effective =
        match olymp_rbac::cache::RbacCache::get_or_build(&state.db, &state.redis, user_id).await {
            Ok(p) => p,
            Err(_) => {
                // No roles assigned — empty permissions
                olymp_rbac::EffectivePermissions {
                    user_id,
                    assignments: vec![],
                }
            }
        };

    // Build AuthContext
    let mut all_permissions = HashSet::new();
    let mut roles = Vec::new();
    let mut scoped_events = Vec::new();

    for assignment in &effective.assignments {
        if !roles.contains(&assignment.role) {
            roles.push(assignment.role.clone());
        }
        all_permissions.extend(assignment.permissions.iter().cloned());
        if let Some(eid) = assignment.event_id {
            if !scoped_events.contains(&eid) {
                scoped_events.push(eid);
            }
        }
    }

    let auth_ctx = AuthContext {
        user_id,
        roles,
        permissions: all_permissions,
        is_global: effective.is_global(),
        scoped_events,
    };

    request.extensions_mut().insert(auth_ctx);
    next.run(request).await
}
