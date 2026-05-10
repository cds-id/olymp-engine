use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::repository::CertificateRepository;
use olymp_core::auth::AuthContext;
use olymp_core::response::{ApiResponse, WithStatus};

// ─── Templates ───

#[utoipa::path(
    post,
    path = "/api/events/{event_id}/certificates/templates",
    tag = "certificates",
    params(("event_id" = Uuid, Path, description = "Event ID")),
    request_body = CreateTemplateRequest,
    responses(
        (status = 201, description = "Template created", body = inline(ApiResponse<CertificateTemplate>)),
    )
)]
pub async fn create_template(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Json(req): Json<CreateTemplateRequest>,
) -> Response {
    if let Err(e) = auth.require("certificate.generate") {
        return e.into_response();
    }
    match CertificateRepository::create_template(&pool, event_id, &req).await {
        Ok(t) => WithStatus(StatusCode::CREATED, ApiResponse::success(t)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/events/{event_id}/certificates/templates",
    tag = "certificates",
    params(("event_id" = Uuid, Path, description = "Event ID")),
    responses(
        (status = 200, description = "List of templates", body = inline(ApiResponse<Vec<CertificateTemplate>>)),
    )
)]
pub async fn list_templates(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("certificate.view") {
        return e.into_response();
    }
    match CertificateRepository::list_templates(&pool, event_id).await {
        Ok(list) => ApiResponse::success(list).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/certificates/templates/{template_id}",
    tag = "certificates",
    params(("template_id" = Uuid, Path, description = "Template ID")),
    responses(
        (status = 200, description = "Template details", body = inline(ApiResponse<CertificateTemplate>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_template(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(template_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("certificate.view") {
        return e.into_response();
    }
    match CertificateRepository::get_template(&pool, template_id).await {
        Ok(Some(t)) => ApiResponse::success(t).into_response(),
        Ok(None) => {
            olymp_core::AppError::NotFound("Template not found".into()).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/certificates/templates/{template_id}",
    tag = "certificates",
    params(("template_id" = Uuid, Path, description = "Template ID")),
    request_body = UpdateTemplateRequest,
    responses(
        (status = 200, description = "Template updated", body = inline(ApiResponse<CertificateTemplate>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_template(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(template_id): Path<Uuid>,
    Json(req): Json<UpdateTemplateRequest>,
) -> Response {
    if let Err(e) = auth.require("certificate.generate") {
        return e.into_response();
    }
    match CertificateRepository::update_template(&pool, template_id, &req).await {
        Ok(t) => ApiResponse::success(t).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Certificate Generation ───

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/certificates/generate",
    tag = "certificates",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    request_body = GenerateCertificatesRequest,
    responses(
        (status = 201, description = "Certificates generated", body = inline(ApiResponse<GenerationResult>)),
        (status = 404, description = "No template for stage")
    )
)]
pub async fn generate_certificates(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Json(req): Json<GenerateCertificatesRequest>,
) -> Response {
    if let Err(e) = auth.require("certificate.generate") {
        return e.into_response();
    }
    match CertificateRepository::generate_for_stage(&pool, stage_id, &req).await {
        Ok(result) => {
            WithStatus(StatusCode::CREATED, ApiResponse::success(result)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

// ─── Certificate Queries ───

#[utoipa::path(
    get,
    path = "/api/participants/{participant_id}/certificates",
    tag = "certificates",
    params(("participant_id" = Uuid, Path, description = "Participant ID")),
    responses(
        (status = 200, description = "List of certificates for participant", body = inline(ApiResponse<Vec<Certificate>>)),
    )
)]
pub async fn list_participant_certificates(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(participant_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("certificate.view") {
        return e.into_response();
    }
    // Ownership: peserta can only view own certificates
    if !auth.is_staff() {
        let owner = sqlx::query_scalar::<_, uuid::Uuid>(
            "SELECT user_id FROM participants WHERE id = $1",
        )
        .bind(participant_id)
        .fetch_optional(&pool)
        .await;
        match owner {
            Ok(Some(uid)) if uid == auth.user_id => {}
            Ok(Some(_)) => {
                return olymp_core::AppError::NotFound("Participant not found".into())
                    .into_response();
            }
            Ok(None) => {
                return olymp_core::AppError::NotFound("Participant not found".into())
                    .into_response();
            }
            Err(e) => return olymp_core::AppError::Database(e).into_response(),
        }
    }
    match CertificateRepository::list_by_participant(&pool, participant_id).await {
        Ok(list) => ApiResponse::success(list).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/certificates/{certificate_id}",
    tag = "certificates",
    params(("certificate_id" = Uuid, Path, description = "Certificate ID")),
    responses(
        (status = 200, description = "Certificate details", body = inline(ApiResponse<Certificate>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_certificate(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(certificate_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("certificate.view") {
        return e.into_response();
    }
    match CertificateRepository::get_certificate(&pool, certificate_id).await {
        Ok(Some(c)) => {
            // Ownership: peserta can only view own certificates
            if !auth.is_staff() {
                let owner = sqlx::query_scalar::<_, uuid::Uuid>(
                    "SELECT p.user_id FROM participants p
                     JOIN participant_stages ps ON ps.participant_id = p.id
                     WHERE ps.id = $1",
                )
                .bind(c.participant_stage_id)
                .fetch_optional(&pool)
                .await;
                match owner {
                    Ok(Some(uid)) if uid == auth.user_id => {}
                    _ => {
                        return olymp_core::AppError::NotFound("Certificate not found".into())
                            .into_response();
                    }
                }
            }
            ApiResponse::success(c).into_response()
        }
        Ok(None) => {
            olymp_core::AppError::NotFound("Certificate not found".into()).into_response()
        }
        Err(e) => e.into_response(),
    }
}

// ─── Event Finalization ───

#[utoipa::path(
    post,
    path = "/api/events/{event_id}/finalize",
    tag = "events",
    params(("event_id" = Uuid, Path, description = "Event ID")),
    responses(
        (status = 200, description = "Event finalized"),
        (status = 400, description = "Not all stages in terminal state")
    )
)]
pub async fn finalize_event(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("olympiad.master.update") {
        return e.into_response();
    }
    match CertificateRepository::finalize_event(&pool, event_id).await {
        Ok(()) => ApiResponse::success(
            serde_json::json!({"status": "finalized", "event_id": event_id}),
        )
        .into_response(),
        Err(e) => e.into_response(),
    }
}
