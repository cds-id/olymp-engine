use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum_extra::headers::{Authorization, authorization::Bearer};
use axum_extra::TypedHeader;
use olymp_core::error::AppError;
use uuid::Uuid;

use crate::jwt::JwtService;

/// Check if a token JTI is in the Redis denylist
async fn is_token_revoked(redis: &redis::Client, jti: &str) -> bool {
    if let Ok(mut conn) = redis.get_multiplexed_async_connection().await {
        let key = format!("jwt:deny:{}", jti);
        redis::cmd("EXISTS")
            .arg(&key)
            .query_async::<i32>(&mut conn)
            .await
            .unwrap_or(0)
            > 0
    } else {
        // Redis down → allow (fail open for availability)
        false
    }
}

/// Add token JTI to Redis denylist with TTL matching token expiry
pub async fn revoke_token(redis: &redis::Client, jti: &str, ttl_secs: u64) -> Result<(), AppError> {
    let mut conn = redis.get_multiplexed_async_connection().await
        .map_err(AppError::Redis)?;
    let key = format!("jwt:deny:{}", jti);
    redis::cmd("SET")
        .arg(&key)
        .arg("1")
        .arg("EX")
        .arg(ttl_secs)
        .query_async::<()>(&mut conn)
        .await
        .map_err(AppError::Redis)?;
    Ok(())
}

/// Extractor for authenticated user (required)
pub struct AuthUser(pub Uuid);

impl FromRequestParts<crate::AppState> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &crate::AppState) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = 
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::Unauthorized.into_response())?;

        let secret = dotenvy::var("OLYMP__AUTH__JWT_SECRET")
            .unwrap_or_else(|_| "default-secret-change-me".to_string());
        
        let jwt_service = JwtService::new(secret, 900);
        
        match jwt_service.verify(bearer.token()) {
            Ok(claims) => {
                // Check denylist
                if is_token_revoked(&state.redis, &claims.jti).await {
                    return Err(AppError::Unauthorized.into_response());
                }
                let user_id = Uuid::parse_str(&claims.sub)
                    .map_err(|_| AppError::Unauthorized.into_response())?;
                Ok(AuthUser(user_id))
            }
            Err(_) => Err(AppError::Unauthorized.into_response()),
        }
    }
}

/// Extractor for optional authenticated user (guest or logged-in)
pub struct OptionalAuthUser(pub Option<Uuid>);

impl FromRequestParts<crate::AppState> for OptionalAuthUser {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &crate::AppState) -> Result<Self, Self::Rejection> {
        let auth_header = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await;
        
        match auth_header {
            Ok(TypedHeader(Authorization(bearer))) => {
                let secret = dotenvy::var("OLYMP__AUTH__JWT_SECRET")
                    .unwrap_or_else(|_| "default-secret-change-me".to_string());
                
                let jwt_service = JwtService::new(secret, 900);
                
                match jwt_service.verify(bearer.token()) {
                    Ok(claims) => {
                        // Check denylist
                        if is_token_revoked(&state.redis, &claims.jti).await {
                            return Ok(OptionalAuthUser(None));
                        }
                        if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                            Ok(OptionalAuthUser(Some(user_id)))
                        } else {
                            Ok(OptionalAuthUser(None))
                        }
                    }
                    Err(_) => Ok(OptionalAuthUser(None)),
                }
            }
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}
