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
use olymp_core::auth::AuthContext;
use olymp_core::response::{ApiResponse, Meta, WithStatus};
use olymp_core::types::ListParams;
use olymp_core::AppError;

// ─── List participants by event ───

#[utoipa::path(
    get,
    path = "/api/events/{event_id}/participants",
    tag = "participants",
    params(
        ("event_id" = Uuid, Path, description = "Event ID"),
        ListParams,
    ),
    responses(
        (status = 200, description = "Paginated list of participants", body = inline(ApiResponse<Vec<Participant>>)),
    )
)]
pub async fn list_event_participants(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Response {
    if let Err(e) = auth.require("participant.view") {
        return e.into_response();
    }
    let total = match ParticipantRepository::count_by_event(&pool, event_id).await {
        Ok(t) => t,
        Err(e) => return e.into_response(),
    };
    match ParticipantRepository::list_by_event(&pool, event_id, params.limit(), params.offset())
        .await
    {
        Ok(participants) => ApiResponse::success(participants)
            .with_meta(Meta::paginated(
                params.page(),
                params.per_page(),
                total as u64,
            ))
            .into_response(),
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
        (status = 201, description = "Participant registered", body = inline(ApiResponse<Participant>)),
        (status = 409, description = "Already registered"),
    )
)]
pub async fn register_participant(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Json(req): Json<RegisterParticipantRequest>,
) -> Response {
    if let Err(e) = auth.require("participant.create") {
        return e.into_response();
    }

    // Check registration window on first stage (district tier)
    let first_stage = match sqlx::query_as::<_, olymp_event::models::Stage>(
        "SELECT * FROM stages WHERE event_id = $1 ORDER BY sequence ASC LIMIT 1",
    )
    .bind(event_id)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return AppError::BadRequest("Event has no stages configured".into()).into_response();
        }
        Err(e) => return AppError::Database(e).into_response(),
    };

    let now = chrono::Utc::now();
    if let Some(opens) = first_stage.registration_opens_at {
        if now < opens {
            return AppError::BadRequest("Registration has not opened yet".into()).into_response();
        }
    }
    if let Some(closes) = first_stage.registration_closes_at {
        if now > closes {
            return AppError::BadRequest("Registration has closed".into()).into_response();
        }
    }

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
        (status = 200, description = "Participant detail with stages", body = inline(ApiResponse<ParticipantDetail>)),
        (status = 404, description = "Not found"),
    )
)]
pub async fn get_participant(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("participant.view") {
        return e.into_response();
    }
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
        (status = 200, description = "Participant updated", body = inline(ApiResponse<Participant>)),
        (status = 404, description = "Not found"),
    )
)]
pub async fn update_participant(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateParticipantRequest>,
) -> Response {
    if let Err(e) = auth.require("participant.update") {
        return e.into_response();
    }
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
        (status = 200, description = "Participant verified", body = inline(ApiResponse<ParticipantStage>)),
        (status = 400, description = "Invalid transition"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn verify_participant(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("participant.verify") {
        return e.into_response();
    }
    transition_first_stage(&pool, id, "verified").await
}

// ─── Approve participant (verified → assigned_to_exam) ───

#[utoipa::path(
    post,
    path = "/api/participants/{id}/approve",
    tag = "participants",
    params(("id" = Uuid, Path, description = "Participant ID")),
    responses(
        (status = 200, description = "Participant approved", body = inline(ApiResponse<ParticipantStage>)),
        (status = 400, description = "Invalid transition"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn approve_participant(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("participant.approve") {
        return e.into_response();
    }
    transition_first_stage(&pool, id, "assigned_to_exam").await
}

// ─── Reject participant (→ disqualified) ───

#[utoipa::path(
    post,
    path = "/api/participants/{id}/reject",
    tag = "participants",
    params(("id" = Uuid, Path, description = "Participant ID")),
    responses(
        (status = 200, description = "Participant rejected", body = inline(ApiResponse<ParticipantStage>)),
        (status = 400, description = "Invalid transition"),
        (status = 404, description = "Not found"),
    )
)]
pub async fn reject_participant(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("participant.reject") {
        return e.into_response();
    }
    transition_first_stage(&pool, id, "disqualified").await
}

// ─── List participants by stage ───

#[utoipa::path(
    get,
    path = "/api/stages/{stage_id}/participants",
    tag = "participants",
    params(
        ("stage_id" = Uuid, Path, description = "Stage ID"),
        ListParams,
    ),
    responses(
        (status = 200, description = "Paginated list of participants for stage", body = inline(ApiResponse<Vec<ParticipantListItem>>)),
    )
)]
pub async fn list_stage_participants(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Response {
    if let Err(e) = auth.require("participant.view") {
        return e.into_response();
    }
    let total = match ParticipantRepository::count_by_stage(&pool, stage_id).await {
        Ok(t) => t,
        Err(e) => return e.into_response(),
    };
    match ParticipantRepository::list_by_stage(&pool, stage_id, params.limit(), params.offset())
        .await
    {
        Ok(items) => ApiResponse::success(items)
            .with_meta(Meta::paginated(
                params.page(),
                params.per_page(),
                total as u64,
            ))
            .into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Batch operations (admin per-stage) ───

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/participants/batch-verify",
    tag = "participants",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    request_body = BatchParticipantRequest,
    responses(
        (status = 200, description = "Batch verify result", body = inline(ApiResponse<BatchResult>)),
    )
)]
pub async fn batch_verify(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Json(req): Json<BatchParticipantRequest>,
) -> Response {
    if let Err(e) = auth.require("participant.verify") {
        return e.into_response();
    }
    match ParticipantRepository::batch_transition(
        &pool,
        stage_id,
        req.participant_ids.as_deref(),
        "registered",
        "verified",
    )
    .await
    {
        Ok(result) => ApiResponse::success(result).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/participants/batch-approve",
    tag = "participants",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    request_body = BatchParticipantRequest,
    responses(
        (status = 200, description = "Batch approve result", body = inline(ApiResponse<BatchResult>)),
    )
)]
pub async fn batch_approve(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Json(req): Json<BatchParticipantRequest>,
) -> Response {
    if let Err(e) = auth.require("participant.approve") {
        return e.into_response();
    }
    match ParticipantRepository::batch_transition(
        &pool,
        stage_id,
        req.participant_ids.as_deref(),
        "verified",
        "assigned_to_exam",
    )
    .await
    {
        Ok(result) => ApiResponse::success(result).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/participants/batch-reject",
    tag = "participants",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    request_body = BatchParticipantRequest,
    responses(
        (status = 200, description = "Batch reject result", body = inline(ApiResponse<BatchResult>)),
    )
)]
pub async fn batch_reject(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Json(req): Json<BatchParticipantRequest>,
) -> Response {
    if let Err(e) = auth.require("participant.reject") {
        return e.into_response();
    }
    // Reject can apply from any non-terminal status
    // Use registered as from_status for bulk, but also allow verified
    let ids = req.participant_ids.as_deref();

    // Try from multiple eligible statuses
    let mut total_affected = 0i32;
    let mut total_skipped = 0i32;
    let mut all_errors = Vec::new();

    for from in &["registered", "verified", "assigned_to_exam"] {
        match ParticipantRepository::batch_transition(
            &pool, stage_id, ids, from, "disqualified",
        )
        .await
        {
            Ok(r) => {
                total_affected += r.affected;
                total_skipped += r.skipped;
                all_errors.extend(r.errors);
            }
            Err(e) => return e.into_response(),
        }
    }

    ApiResponse::success(BatchResult {
        affected: total_affected,
        skipped: total_skipped,
        errors: all_errors,
    })
    .into_response()
}

// ─── My participations (peserta self-service) ───

#[utoipa::path(
    get,
    path = "/api/users/me/participations",
    tag = "participants",
    responses(
        (status = 200, description = "Current user's event registrations", body = inline(ApiResponse<Vec<Participant>>)),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn my_participations(
    auth: AuthContext,
    State(pool): State<PgPool>,
) -> Response {
    if let Err(e) = auth.require("participant.view") {
        return e.into_response();
    }
    match ParticipantRepository::list_by_user(&pool, auth.user_id).await {
        Ok(list) => ApiResponse::success(list).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Helper: transition first/current stage ───

async fn transition_first_stage(
    pool: &PgPool,
    participant_id: Uuid,
    new_status: &str,
) -> Response {
    let stages = match ParticipantRepository::get_participant_stages(pool, participant_id).await {
        Ok(s) => s,
        Err(e) => return e.into_response(),
    };

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
