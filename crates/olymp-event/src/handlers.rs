use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::repository::EventRepository;
use olymp_core::response::{ApiResponse, WithStatus};
use olymp_core::AppError;

// ─── Education Levels ───

#[utoipa::path(
    get,
    path = "/api/education-levels",
    tag = "events",
    responses((status = 200, description = "List of education levels"))
)]
pub async fn list_education_levels(State(pool): State<PgPool>) -> Response {
    match EventRepository::list_education_levels(&pool).await {
        Ok(levels) => ApiResponse::success(levels).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/education-levels",
    tag = "events",
    request_body = CreateEducationLevelRequest,
    responses(
        (status = 201, description = "Education level created"),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_education_level(
    State(pool): State<PgPool>,
    Json(req): Json<CreateEducationLevelRequest>,
) -> Response {
    match EventRepository::create_education_level(&pool, &req.name).await {
        Ok(level) => WithStatus(StatusCode::CREATED, ApiResponse::success(level)).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Subjects ───

#[utoipa::path(
    get,
    path = "/api/subjects",
    tag = "events",
    responses((status = 200, description = "List of subjects"))
)]
pub async fn list_subjects(State(pool): State<PgPool>) -> Response {
    match EventRepository::list_subjects(&pool).await {
        Ok(subjects) => ApiResponse::success(subjects).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/subjects",
    tag = "events",
    request_body = CreateSubjectRequest,
    responses(
        (status = 201, description = "Subject created"),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_subject(
    State(pool): State<PgPool>,
    Json(req): Json<CreateSubjectRequest>,
) -> Response {
    match EventRepository::create_subject(&pool, &req.name).await {
        Ok(subject) => WithStatus(StatusCode::CREATED, ApiResponse::success(subject)).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Events ───

#[utoipa::path(
    get,
    path = "/api/events",
    tag = "events",
    responses((status = 200, description = "List of events"))
)]
pub async fn list_events(State(pool): State<PgPool>) -> Response {
    match EventRepository::list_events(&pool).await {
        Ok(events) => ApiResponse::success(events).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/events/{id}",
    tag = "events",
    params(("id" = Uuid, Path, description = "Event ID")),
    responses(
        (status = 200, description = "Event details"),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_event(State(pool): State<PgPool>, Path(id): Path<Uuid>) -> Response {
    match EventRepository::get_event(&pool, id).await {
        Ok(Some(event)) => ApiResponse::success(event).into_response(),
        Ok(None) => AppError::NotFound("Event not found".into()).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/events",
    tag = "events",
    request_body = CreateEventRequest,
    responses(
        (status = 201, description = "Event created"),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_event(
    State(pool): State<PgPool>,
    Json(req): Json<CreateEventRequest>,
) -> Response {
    match EventRepository::create_event(&pool, &req.name, &req.academic_year).await {
        Ok(event) => WithStatus(StatusCode::CREATED, ApiResponse::success(event)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/events/{id}",
    tag = "events",
    params(("id" = Uuid, Path, description = "Event ID")),
    request_body = UpdateEventRequest,
    responses(
        (status = 200, description = "Event updated"),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_event(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateEventRequest>,
) -> Response {
    match EventRepository::update_event(&pool, id, &req).await {
        Ok(event) => ApiResponse::success(event).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Stages ───

#[utoipa::path(
    get,
    path = "/api/events/{event_id}/stages",
    tag = "events",
    params(("event_id" = Uuid, Path, description = "Event ID")),
    responses((status = 200, description = "List of stages for event"))
)]
pub async fn list_stages(
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
) -> Response {
    match EventRepository::list_stages(&pool, event_id).await {
        Ok(stages) => ApiResponse::success(stages).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/events/{event_id}/stages",
    tag = "events",
    params(("event_id" = Uuid, Path, description = "Event ID")),
    request_body = CreateStageRequest,
    responses(
        (status = 201, description = "Stage created"),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_stage(
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Json(req): Json<CreateStageRequest>,
) -> Response {
    match EventRepository::create_stage(&pool, event_id, req.tier).await {
        Ok(stage) => WithStatus(StatusCode::CREATED, ApiResponse::success(stage)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/stages/{id}/status",
    tag = "events",
    params(("id" = Uuid, Path, description = "Stage ID")),
    request_body = UpdateStageStatusRequest,
    responses(
        (status = 200, description = "Stage status updated"),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_stage_status(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStageStatusRequest>,
) -> Response {
    match EventRepository::update_stage_status(&pool, id, req.status).await {
        Ok(stage) => ApiResponse::success(stage).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Event Categories ───

#[utoipa::path(
    get,
    path = "/api/events/{event_id}/categories",
    tag = "events",
    params(("event_id" = Uuid, Path, description = "Event ID")),
    responses((status = 200, description = "List of event categories"))
)]
pub async fn list_event_categories(
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
) -> Response {
    match EventRepository::list_event_categories(&pool, event_id).await {
        Ok(cats) => ApiResponse::success(cats).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/events/{event_id}/categories",
    tag = "events",
    params(("event_id" = Uuid, Path, description = "Event ID")),
    request_body = CreateEventCategoryRequest,
    responses(
        (status = 201, description = "Event category created"),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_event_category(
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Json(req): Json<CreateEventCategoryRequest>,
) -> Response {
    match EventRepository::create_event_category(
        &pool,
        event_id,
        req.education_level_id,
        req.subject_id,
    )
    .await
    {
        Ok(cat) => WithStatus(StatusCode::CREATED, ApiResponse::success(cat)).into_response(),
        Err(e) => e.into_response(),
    }
}
