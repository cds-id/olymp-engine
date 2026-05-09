use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use sqlx::PgPool;
use crate::models::{RegisterParticipantRequest, ParticipantResponse, AdvanceTierRequest};
use crate::service::ParticipantService;

pub async fn register_participant(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterParticipantRequest>,
) -> Result<(StatusCode, Json<ParticipantResponse>), (StatusCode, String)> {
    let user_id = Uuid::new_v4();

    let participant = ParticipantService::register_participant(&pool, user_id, req)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok((
        StatusCode::CREATED,
        Json(ParticipantResponse {
            id: participant.id,
            current_tier: participant.current_tier,
            is_locked: participant.is_locked,
            score: participant.score,
            rank: participant.rank,
        }),
    ))
}

pub async fn get_profile(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ParticipantResponse>, (StatusCode, String)> {
    let participant = ParticipantService::get_participant_profile(&pool, user_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    Ok(Json(ParticipantResponse {
        id: participant.id,
        current_tier: participant.current_tier,
        is_locked: participant.is_locked,
        score: participant.score,
        rank: participant.rank,
    }))
}

pub async fn advance_tier(
    State(pool): State<PgPool>,
    Path(participant_id): Path<Uuid>,
    Json(req): Json<AdvanceTierRequest>,
) -> Result<Json<ParticipantResponse>, (StatusCode, String)> {
    let participant = ParticipantService::advance_tier(&pool, participant_id, req)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(ParticipantResponse {
        id: participant.id,
        current_tier: participant.current_tier,
        is_locked: participant.is_locked,
        score: participant.score,
        rank: participant.rank,
    }))
}
