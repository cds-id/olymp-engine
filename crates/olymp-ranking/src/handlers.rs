use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::repository::RankingRepository;
use olymp_core::response::{ApiResponse, WithStatus};

// ─── Ranking Rules ───

#[utoipa::path(
    get,
    path = "/api/stages/{stage_id}/ranking/rules",
    tag = "ranking",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    responses(
        (status = 200, description = "Ranking rule for stage", body = RankingRule),
        (status = 404, description = "No rule configured")
    )
)]
pub async fn get_ranking_rule(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
) -> Response {
    match RankingRepository::get_rule_by_stage(&pool, stage_id).await {
        Ok(Some(rule)) => ApiResponse::success(rule).into_response(),
        Ok(None) => {
            olymp_core::AppError::NotFound("No ranking rule for this stage".into()).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/ranking/rules",
    tag = "ranking",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    request_body = CreateRankingRuleRequest,
    responses(
        (status = 200, description = "Ranking rule created/updated", body = RankingRule),
    )
)]
pub async fn upsert_ranking_rule(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Json(req): Json<CreateRankingRuleRequest>,
) -> Response {
    match RankingRepository::upsert_rule(&pool, stage_id, &req).await {
        Ok(rule) => ApiResponse::success(rule).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Calculate ───

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/ranking/calculate",
    tag = "ranking",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    responses(
        (status = 201, description = "Ranking calculated", body = RankingResult),
        (status = 400, description = "No participants or no rule"),
        (status = 404, description = "Rule not found")
    )
)]
pub async fn calculate_ranking(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
) -> Response {
    match RankingRepository::calculate(&pool, stage_id).await {
        Ok(result) => {
            WithStatus(StatusCode::CREATED, ApiResponse::success(result)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

// ─── View Ranking ───

#[utoipa::path(
    get,
    path = "/api/stages/{stage_id}/ranking",
    tag = "ranking",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    responses(
        (status = 200, description = "Latest ranking with entries", body = RankingResultWithEntries),
        (status = 404, description = "No ranking calculated yet")
    )
)]
pub async fn get_ranking(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
) -> Response {
    let result = match RankingRepository::get_latest_result(&pool, stage_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return olymp_core::AppError::NotFound("No ranking result for this stage".into())
                .into_response()
        }
        Err(e) => return e.into_response(),
    };

    match RankingRepository::get_entries(&pool, result.id).await {
        Ok(entries) => {
            ApiResponse::success(RankingResultWithEntries { result, entries }).into_response()
        }
        Err(e) => e.into_response(),
    }
}

// ─── Review / Approve / Publish ───

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/ranking/review",
    tag = "ranking",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    request_body = ReviewRequest,
    responses(
        (status = 200, description = "Ranking reviewed (draft → reviewed)", body = RankingResult),
        (status = 400, description = "Invalid transition")
    )
)]
pub async fn review_ranking(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Json(req): Json<ReviewRequest>,
) -> Response {
    let result = match RankingRepository::get_latest_result(&pool, stage_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return olymp_core::AppError::NotFound("No ranking result".into()).into_response()
        }
        Err(e) => return e.into_response(),
    };
    match RankingRepository::transition_result(&pool, result.id, "reviewed", Some(req.actor_id))
        .await
    {
        Ok(r) => ApiResponse::success(r).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/ranking/approve",
    tag = "ranking",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    request_body = ApproveRequest,
    responses(
        (status = 200, description = "Ranking approved (reviewed → approved)", body = RankingResult),
        (status = 400, description = "Invalid transition")
    )
)]
pub async fn approve_ranking(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Json(req): Json<ApproveRequest>,
) -> Response {
    let result = match RankingRepository::get_latest_result(&pool, stage_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return olymp_core::AppError::NotFound("No ranking result".into()).into_response()
        }
        Err(e) => return e.into_response(),
    };
    match RankingRepository::transition_result(&pool, result.id, "approved", Some(req.actor_id))
        .await
    {
        Ok(r) => ApiResponse::success(r).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/ranking/publish",
    tag = "ranking",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    responses(
        (status = 200, description = "Ranking published (approved → published)", body = RankingResult),
        (status = 400, description = "Invalid transition")
    )
)]
pub async fn publish_ranking(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
) -> Response {
    let result = match RankingRepository::get_latest_result(&pool, stage_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return olymp_core::AppError::NotFound("No ranking result".into()).into_response()
        }
        Err(e) => return e.into_response(),
    };
    match RankingRepository::transition_result(&pool, result.id, "published", None).await {
        Ok(r) => ApiResponse::success(r).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Promotion ───

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/promote",
    tag = "ranking",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    responses(
        (status = 200, description = "Qualified participants promoted to next stage", body = PromotionResult),
        (status = 400, description = "Ranking not approved"),
        (status = 404, description = "No next stage")
    )
)]
pub async fn promote(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
) -> Response {
    match RankingRepository::promote(&pool, stage_id).await {
        Ok(result) => ApiResponse::success(result).into_response(),
        Err(e) => e.into_response(),
    }
}
