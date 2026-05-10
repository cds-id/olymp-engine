use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::repository::ParticipantRepository;
use olymp_core::response::{ApiResponse, WithStatus};


// ─── List participants by event ───

#[derive(Debug, serde::Deserialize)]
pub struct ListParams {
    pub per_page: Option<i64>,
    pub offset: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/events/{event_id}/participants",
    tag = "participants",
    params(
        ("event_id" = Uuid, Path, description = "Event ID"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("offset" = Option<i64>, Query, description = "Offset"),
    ),
    responses(
        (status = 200, description = "List of participants", body = Vec<Participant>),
    )
)]
pub async fn list_event_participants(
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Response {
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    match ParticipantRepository::list_by_event(&pool, event_id, per_page, offset).await {
        Ok(participants) => ApiResponse::success(participants).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Register participant ───

#[utoipa::path(
    post,
    path = "/api/events/{event_id}/participants",
    tag = "participants",
    params(("event_id" = Uuid, Path, description = "Event ID")),
    request_body = RegisterParticipantRequest,
    responses(
        (status = 201, description = "Participant registered", body = Participant),
        (status = 409, description = "Already registered"),
    )
)]
pub async fn register_participant(
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Json(req): Json<RegisterParticipantRequest>,
) -> Response {
    match ParticipantRepository::register(&pool, event_id, &req).await {
        Ok(participant) => {
            WithStatus(StatusCode::CREATED, ApiResponse::success(participant)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

// ─── Get participant detail ───

#[utoipa::path(
    get,
    path = "/api/participants/{id}",
    tag = "participants",
    params(("id" = Uuid, Path, description = "Participant ID")),
    responses(
        (status = 200, description = "Participant detail with stages", body = ParticipantDetail),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_participant(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Response {
    let participant = match ParticipantRepository::get_by_id(&pool, id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return olymp_core::AppError::NotFound("Participant not found".into()).into_response()
        }
        Err(e) => return e.into_response(),
    };

    let stages = match ParticipantRepository::get_participant_stages(&pool, id).await {
        Ok(s) => s,
        Err(e) => return e.into_response(),
    };

    ApiResponse::success(ParticipantDetail {
        participant,
        stages,
    })
    .into_response()
}

// ─── Update participant ───

#[utoipa::path(
    put,
    path = "/api/participants/{id}",
    tag = "participants",
    params(("id" = Uuid, Path, description = "Participant ID")),
    request_body = UpdateParticipantRequest,
    responses(
        (status = 200, description = "Participant updated", body = Participant),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_participant(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateParticipantRequest>,
) -> Response {
    match ParticipantRepository::update(&pool, id, &req).await {
        Ok(p) => ApiResponse::success(p).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Verify participant (registered → verified) ───

#[utoipa::path(
    post,
    path = "/api/participants/{id}/verify",
    tag = "participants",
    params(("id" = Uuid, Path, description = "Participant ID")),
    responses(
        (status = 200, description = "Participant verified", body = ParticipantStage),
        (status = 400, description = "Invalid transition"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn verify_participant(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Response {
    transition_first_stage(&pool, id, "verified").await
}

// ─── Approve participant (verified → assigned_to_exam) ───

#[utoipa::path(
    post,
    path = "/api/participants/{id}/approve",
    tag = "participants",
    params(("id" = Uuid, Path, description = "Participant ID")),
    responses(
        (status = 200, description = "Participant approved", body = ParticipantStage),
        (status = 400, description = "Invalid transition"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn approve_participant(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Response {
    transition_first_stage(&pool, id, "assigned_to_exam").await
}

// ─── Reject participant (→ disqualified) ───

#[utoipa::path(
    post,
    path = "/api/participants/{id}/reject",
    tag = "participants",
    params(("id" = Uuid, Path, description = "Participant ID")),
    responses(
        (status = 200, description = "Participant rejected", body = ParticipantStage),
        (status = 400, description = "Invalid transition"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn reject_participant(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Response {
    transition_first_stage(&pool, id, "disqualified").await
}

// ─── List participants by stage ───

#[utoipa::path(
    get,
    path = "/api/stages/{stage_id}/participants",
    tag = "participants",
    params(
        ("stage_id" = Uuid, Path, description = "Stage ID"),
        ("per_page" = Option<i64>, Query, description = "Items per page"),
        ("offset" = Option<i64>, Query, description = "Offset"),
    ),
    responses(
        (status = 200, description = "List of participants for stage", body = Vec<ParticipantListItem>),
    )
)]
pub async fn list_stage_participants(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Response {
    let per_page = params.per_page.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    match ParticipantRepository::list_by_stage(&pool, stage_id, per_page, offset).await {
        Ok(items) => ApiResponse::success(items).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Helper: transition first/current stage ───

async fn transition_first_stage(pool: &PgPool, participant_id: Uuid, new_status: &str) -> Response {
    // Get participant's stages, find first actionable one
    let stages = match ParticipantRepository::get_participant_stages(pool, participant_id).await {
        Ok(s) => s,
        Err(e) => return e.into_response(),
    };

    // Find stage that can transition to new_status
    let stage = stages.iter().find(|s| s.can_transition_to(new_status));

    match stage {
        Some(s) => {
            match ParticipantRepository::transition_stage_status(pool, s.id, new_status).await {
                Ok(updated) => ApiResponse::success(updated).into_response(),
                Err(e) => e.into_response(),
            }
        }
        None => {
            if stages.is_empty() {
                olymp_core::AppError::NotFound("No stage entries for participant".into())
                    .into_response()
            } else {
                olymp_core::AppError::BadRequest(format!(
                    "No stage entry can transition to '{}'",
                    new_status
                ))
                .into_response()
            }
        }
    }
}
