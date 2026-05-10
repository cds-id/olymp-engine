use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

use olymp_core::error::AppError;
use olymp_core::response::{ApiResponse, WithStatus};
use olymp_notification::models::MagicLinkData;
use crate::AppState;
use crate::extractors::AuthUser;
use crate::notifications::{NotificationPreferences, NotificationPrefsService, UpdateNotificationPreferences};
use crate::user::UserProfile;
use crate::{JwtService, MagicLinkService, UserService, PasswordService, RegistrationService};
use olymp_core::auth::AuthContext;
use sqlx;

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MagicLinkRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct MagicLinkResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CallbackRequest {
    pub email: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub username: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct LogoutRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub username: Option<String>,
    pub phone: Option<String>,
}

/// Current user's roles and permissions for frontend conditional UI
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MyRolesResponse {
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub is_staff: bool,
    pub is_admin: bool,
}

#[utoipa::path(
    post,
    path = "/api/auth/magic-link",
    tag = "auth",
    request_body = MagicLinkRequest,
    responses(
        (status = 200, description = "Magic link sent", body = inline(ApiResponse<MagicLinkResponse>)),
        (status = 400, description = "Bad request")
    )
)]
pub async fn request_magic_link(
    State(state): State<AppState>,
    Json(req): Json<MagicLinkRequest>,
) -> Response {
    let magic_link_service = MagicLinkService::new(state.db.clone(), 900);

    match magic_link_service.create(&req.email).await {
        Ok(token) => {
            // Build magic link URL
            let base_url = dotenvy::var("OLYMP__APP__BASE_URL")
                .unwrap_or_else(|_| "https://olymp.id".to_string());
            let magic_link = format!("{}/auth/callback?email={}&token={}", base_url, req.email, token);
            
            // Send email if configured
            if let Some(ref email_svc) = state.email {
                let data = MagicLinkData {
                    name: req.email.split('@').next().unwrap_or("User").to_string(),
                    magic_link: magic_link.clone(),
                    expires_in_minutes: 15,
                };
                if let Err(e) = email_svc.send_magic_link(&req.email, data).await {
                    tracing::error!("Failed to send magic link email: {:?}", e);
                    // Don't fail - still return success for testing
                }
            } else {
                tracing::info!("Magic link (no email svc): {}", magic_link);
            }
            
            ApiResponse::success(MagicLinkResponse {
                message: "Magic link sent to email".to_string(),
            }).into_response()
        }
        Err(e) => AppError::BadRequest(e.to_string()).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/callback",
    tag = "auth",
    request_body = CallbackRequest,
    responses(
        (status = 200, description = "Authentication successful", body = inline(ApiResponse<AuthResponse>)),
        (status = 401, description = "Invalid token")
    )
)]
pub async fn magic_link_callback(
    State(state): State<AppState>,
    Json(req): Json<CallbackRequest>,
) -> Response {
    let magic_link_service = MagicLinkService::new(state.db.clone(), 900);
    let user_service = UserService::new(state.db.clone());
    let jwt_service = JwtService::new(state.config.auth.jwt_secret.clone(), 900);

    match magic_link_service.verify(&req.email, &req.token).await {
        Ok(true) => {
            let user_id = match user_service.get_or_create(&req.email).await {
                Ok(id) => id,
                Err(e) => return AppError::Internal(e.to_string()).into_response(),
            };
            let access_token = match jwt_service.issue(user_id, req.email.clone()) {
                Ok(t) => t,
                Err(e) => return AppError::Internal(e.to_string()).into_response(),
            };
            let refresh_token = match jwt_service.issue_refresh(user_id) {
                Ok(t) => t,
                Err(e) => return AppError::Internal(e.to_string()).into_response(),
            };
            ApiResponse::success(AuthResponse {
                access_token,
                refresh_token,
                user_id: user_id.to_string(),
                email: req.email,
            }).into_response()
        }
        Ok(false) => AppError::Unauthorized.into_response(),
        Err(e) => AppError::BadRequest(e.to_string()).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = inline(ApiResponse<AuthResponse>)),
        (status = 400, description = "Validation error")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Response {
    let reg_service = RegistrationService::new(state.db.clone());
    let jwt_service = JwtService::new(state.config.auth.jwt_secret.clone(), 900);

    let user_id = match reg_service
        .register_with_password(&req.email, &req.username, &req.password, req.name.as_deref())
        .await
    {
        Ok(id) => id,
        Err(e) => return AppError::BadRequest(e.to_string()).into_response(),
    };

    let access_token = match jwt_service.issue(user_id, req.email.clone()) {
        Ok(t) => t,
        Err(e) => return AppError::Internal(e.to_string()).into_response(),
    };
    let refresh_token = match jwt_service.issue_refresh(user_id) {
        Ok(t) => t,
        Err(e) => return AppError::Internal(e.to_string()).into_response(),
    };

    // Send welcome email
    if let Some(ref email_svc) = state.email {
        let name = req.name.clone().unwrap_or_else(|| req.username.clone());
        let data = olymp_notification::models::WelcomeData { name };
        if let Err(e) = email_svc.send_welcome(&req.email, data).await {
            tracing::error!("Failed to send welcome email: {:?}", e);
        }
    }

    WithStatus(
        StatusCode::CREATED,
        ApiResponse::success(AuthResponse {
            access_token,
            refresh_token,
            user_id: user_id.to_string(),
            email: req.email,
        }),
    ).into_response()
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = inline(ApiResponse<AuthResponse>)),
        (status = 401, description = "Invalid credentials")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Response {
    let reg_service = RegistrationService::new(state.db.clone());
    let jwt_service = JwtService::new(state.config.auth.jwt_secret.clone(), 900);

    let (user_id, password_hash) = match reg_service.get_by_username(&req.username).await {
        Ok(Some(pair)) => pair,
        Ok(None) => return AppError::Unauthorized.into_response(),
        Err(e) => return AppError::Internal(e.to_string()).into_response(),
    };

    match PasswordService::verify_password(&req.password, &password_hash) {
        Ok(true) => {}
        Ok(false) => return AppError::Unauthorized.into_response(),
        Err(e) => return AppError::Internal(e.to_string()).into_response(),
    }

    let user_service = UserService::new(state.db.clone());
    let email = match user_service.get_by_id(user_id).await {
        Ok(Some((_id, email))) => email,
        _ => return AppError::Unauthorized.into_response(),
    };

    let access_token = match jwt_service.issue(user_id, email.clone()) {
        Ok(t) => t,
        Err(e) => return AppError::Internal(e.to_string()).into_response(),
    };
    let refresh_token = match jwt_service.issue_refresh(user_id) {
        Ok(t) => t,
        Err(e) => return AppError::Internal(e.to_string()).into_response(),
    };

    ApiResponse::success(AuthResponse {
        access_token,
        refresh_token,
        user_id: user_id.to_string(),
        email,
    }).into_response()
}

#[utoipa::path(
    get,
    path = "/api/users/me",
    tag = "auth",
    responses(
        (status = 200, description = "User profile", body = inline(ApiResponse<UserProfile>)),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Response {
    let svc = UserService::new(state.db.clone());
    match svc.get_full_by_id(user_id).await {
        Ok(Some(profile)) => ApiResponse::success(profile).into_response(),
        Ok(None) => AppError::NotFound("User not found".to_string()).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/users/me",
    tag = "auth",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = inline(ApiResponse<UserProfile>)),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn update_me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Response {
    let svc = UserService::new(state.db.clone());
    match svc.update_profile(
        user_id,
        req.name.as_deref(),
        req.username.as_deref(),
        req.phone.as_deref(),
    ).await {
        Ok(profile) => ApiResponse::success(profile).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    tag = "auth",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset email sent"),
        (status = 400, description = "Bad request")
    )
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Response {
    // Always return success to avoid email enumeration
    let user = sqlx::query_as::<_, (uuid::Uuid, String)>(
        "SELECT id, email FROM auth.users WHERE email = $1"
    )
    .bind(&req.email)
    .fetch_optional(&state.db)
    .await;

    if let Ok(Some((user_id, _email))) = user {
        let pwd_svc = PasswordService::new(state.db.clone());
        if let Ok(token) = pwd_svc.create_reset_token(user_id).await {
            let base_url = dotenvy::var("OLYMP__APP__BASE_URL")
                .unwrap_or_else(|_| "https://olymp.id".to_string());
            let reset_link = format!("{}/auth/reset-password?token={}", base_url, token);

            if let Some(ref email_svc) = state.email {
                let data = olymp_notification::models::PasswordResetData {
                    name: req.email.split('@').next().unwrap_or("User").to_string(),
                    reset_link,
                    expires_in_minutes: 60,
                };
                if let Err(e) = email_svc.send_password_reset(&req.email, data).await {
                    tracing::error!("Failed to send password reset email: {:?}", e);
                }
            } else {
                tracing::info!("Password reset link (no email svc): {}", reset_link);
            }
        }
    }

    ApiResponse::success(serde_json::json!({
        "message": "If the email exists, a reset link has been sent"
    })).into_response()
}

#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
    tag = "auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful"),
        (status = 400, description = "Invalid token")
    )
)]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Response {
    let pwd_svc = PasswordService::new(state.db.clone());

    let user_id = match pwd_svc.verify_reset_token(&req.token).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    match pwd_svc.update_password(user_id, &req.new_password).await {
        Ok(()) => ApiResponse::success(serde_json::json!({
            "message": "Password has been reset"
        })).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/users/me/password",
    tag = "auth",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed"),
        (status = 401, description = "Invalid current password")
    ),
    security(("bearer" = []))
)]
pub async fn change_password(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> Response {
    // Get current password hash
    let hash = sqlx::query_scalar::<_, Option<String>>(
        "SELECT password_hash FROM auth.users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await;

    let hash = match hash {
        Ok(Some(Some(h))) => h,
        Ok(Some(None)) => return AppError::BadRequest(
            "Account uses magic link / OAuth, no password set".to_string()
        ).into_response(),
        Ok(None) => return AppError::NotFound("User not found".to_string()).into_response(),
        Err(e) => return AppError::Database(e).into_response(),
    };

    // Verify current password
    match PasswordService::verify_password(&req.current_password, &hash) {
        Ok(true) => {},
        Ok(false) => return AppError::BadRequest("Current password incorrect".to_string()).into_response(),
        Err(e) => return e.into_response(),
    }

    // Update
    let pwd_svc = PasswordService::new(state.db.clone());
    match pwd_svc.update_password(user_id, &req.new_password).await {
        Ok(()) => ApiResponse::success(serde_json::json!({"message": "Password changed"})).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed", body = inline(ApiResponse<AuthResponse>)),
        (status = 401, description = "Invalid refresh token")
    )
)]
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Response {
    let jwt_service = JwtService::new(state.config.auth.jwt_secret.clone(), 900);
    let user_service = UserService::new(state.db.clone());

    let claims = match jwt_service.verify_refresh(&req.refresh_token) {
        Ok(c) => c,
        Err(_) => return AppError::Unauthorized.into_response(),
    };

    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return AppError::Unauthorized.into_response(),
    };

    let email = match user_service.get_by_id(user_id).await {
        Ok(Some((_id, email))) => email,
        _ => return AppError::Unauthorized.into_response(),
    };

    let access_token = match jwt_service.issue(user_id, email.clone()) {
        Ok(t) => t,
        Err(e) => return e.into_response(),
    };
    let refresh_token = match jwt_service.issue_refresh(user_id) {
        Ok(t) => t,
        Err(e) => return e.into_response(),
    };

    ApiResponse::success(AuthResponse {
        access_token,
        refresh_token,
        user_id: user_id.to_string(),
        email,
    }).into_response()
}

#[utoipa::path(
    get,
    path = "/api/users/me/notifications",
    tag = "auth",
    responses(
        (status = 200, description = "Notification preferences", body = inline(ApiResponse<NotificationPreferences>)),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn get_notifications(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Response {
    let svc = NotificationPrefsService::new(state.db.clone());
    match svc.get(user_id).await {
        Ok(prefs) => ApiResponse::success(prefs).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/users/me/notifications",
    tag = "auth",
    request_body = UpdateNotificationPreferences,
    responses(
        (status = 200, description = "Preferences updated", body = inline(ApiResponse<NotificationPreferences>)),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn update_notifications(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<UpdateNotificationPreferences>,
) -> Response {
    let svc = NotificationPrefsService::new(state.db.clone());
    match svc.update(user_id, req).await {
        Ok(prefs) => ApiResponse::success(prefs).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/users/me/roles",
    tag = "auth",
    responses(
        (status = 200, description = "Current user roles and permissions", body = inline(ApiResponse<MyRolesResponse>)),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn my_roles(auth: AuthContext) -> Response {
    let is_staff = auth.is_staff();
    let is_admin = auth.is_admin();
    let mut perms: Vec<String> = auth.permissions.into_iter().collect();
    perms.sort();
    ApiResponse::success(MyRolesResponse {
        roles: auth.roles,
        permissions: perms,
        is_staff,
        is_admin,
    })
    .into_response()
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logged out"),
        (status = 400, description = "Bad request")
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> Response {
    let jwt_service = JwtService::new(state.config.auth.jwt_secret.clone(), 900);

    // Try to revoke access token
    if let Ok(claims) = jwt_service.verify(&req.token) {
        let ttl = (claims.exp - chrono::Utc::now().timestamp()).max(0) as u64;
        if ttl > 0 {
            let _ = crate::extractors::revoke_token(&state.redis, &claims.jti, ttl).await;
        }
    }

    ApiResponse::success(serde_json::json!({"message": "Logged out"})).into_response()
}
