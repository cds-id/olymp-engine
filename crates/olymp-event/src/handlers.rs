use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::repository::EventRepository;
use olymp_core::auth::AuthContext;
use olymp_core::response::{ApiResponse, Meta, WithStatus};
use olymp_core::types::ListParams;
use olymp_core::AppError;

// ─── Education Levels ───

#[utoipa::path(
    get,
    path = "/api/education-levels",
    tag = "events",
    responses((status = 200, description = "List of education levels", body = inline(ApiResponse<Vec<EducationLevel>>)))
)]
pub async fn list_education_levels(auth: AuthContext, State(pool): State<PgPool>) -> Response {
    if let Err(e) = auth.require("region.view") {
        return e.into_response();
    }
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
        (status = 201, description = "Education level created", body = inline(ApiResponse<EducationLevel>)),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_education_level(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Json(req): Json<CreateEducationLevelRequest>,
) -> Response {
    if let Err(e) = auth.require("olympiad.master.create") {
        return e.into_response();
    }
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
    responses((status = 200, description = "List of subjects", body = inline(ApiResponse<Vec<Subject>>)))
)]
pub async fn list_subjects(auth: AuthContext, State(pool): State<PgPool>) -> Response {
    if let Err(e) = auth.require("region.view") {
        return e.into_response();
    }
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
        (status = 201, description = "Subject created", body = inline(ApiResponse<Subject>)),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_subject(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Json(req): Json<CreateSubjectRequest>,
) -> Response {
    if let Err(e) = auth.require("olympiad.master.create") {
        return e.into_response();
    }
    match EventRepository::create_subject(&pool, &req.name).await {
        Ok(subject) => {
            WithStatus(StatusCode::CREATED, ApiResponse::success(subject)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

// ─── Events ───

#[utoipa::path(
    get,
    path = "/api/events",
    tag = "events",
    params(ListParams),
    responses(
        (status = 200, description = "Paginated list of events", body = inline(ApiResponse<Vec<Event>>)),
    )
)]
pub async fn list_events(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Query(params): Query<ListParams>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    let total = match EventRepository::count_events(&pool).await {
        Ok(t) => t,
        Err(e) => return e.into_response(),
    };
    match EventRepository::list_events(&pool, params.limit(), params.offset()).await {
        Ok(events) => ApiResponse::success(events)
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
    get,
    path = "/api/events/{id}",
    tag = "events",
    params(("id" = Uuid, Path, description = "Event ID")),
    responses(
        (status = 200, description = "Event details", body = inline(ApiResponse<Event>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_event(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
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
        (status = 201, description = "Event created", body = inline(ApiResponse<Event>)),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_event(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Json(req): Json<CreateEventRequest>,
) -> Response {
    if let Err(e) = auth.require("olympiad.master.create") {
        return e.into_response();
    }
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
        (status = 200, description = "Event updated", body = inline(ApiResponse<Event>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_event(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateEventRequest>,
) -> Response {
    if let Err(e) = auth.require("olympiad.master.update") {
        return e.into_response();
    }
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
    responses((status = 200, description = "List of stages for event", body = inline(ApiResponse<Vec<Stage>>)))
)]
pub async fn list_stages(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
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
        (status = 201, description = "Stage created", body = inline(ApiResponse<Stage>)),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_stage(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Json(req): Json<CreateStageRequest>,
) -> Response {
    if let Err(e) = auth.require("olympiad.stage.manage") {
        return e.into_response();
    }
    match EventRepository::create_stage(&pool, event_id, &req).await {
        Ok(stage) => WithStatus(StatusCode::CREATED, ApiResponse::success(stage)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/events/{event_id}/stages/available",
    tag = "events",
    params(
        ("event_id" = Uuid, Path, description = "Event ID"),
        AvailableStageFilters,
    ),
    responses(
        (status = 200, description = "Stages open for registration with enrollment counts", body = inline(ApiResponse<Vec<StageWithEnrollment>>)),
    )
)]
pub async fn list_available_stages(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Query(filters): Query<AvailableStageFilters>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    match EventRepository::list_available_stages(&pool, event_id, &filters).await {
        Ok(stages) => ApiResponse::success(stages).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/stages/{id}",
    tag = "events",
    params(("id" = Uuid, Path, description = "Stage ID")),
    responses(
        (status = 200, description = "Stage details", body = inline(ApiResponse<Stage>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_stage(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    match EventRepository::get_stage(&pool, id).await {
        Ok(Some(stage)) => ApiResponse::success(stage).into_response(),
        Ok(None) => AppError::NotFound("Stage not found".into()).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/stages/{id}",
    tag = "events",
    params(("id" = Uuid, Path, description = "Stage ID")),
    request_body = UpdateStageRequest,
    responses(
        (status = 200, description = "Stage updated", body = inline(ApiResponse<Stage>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_stage(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStageRequest>,
) -> Response {
    if let Err(e) = auth.require("olympiad.stage.manage") {
        return e.into_response();
    }
    match EventRepository::update_stage(&pool, id, &req).await {
        Ok(stage) => ApiResponse::success(stage).into_response(),
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
        (status = 200, description = "Stage status updated", body = inline(ApiResponse<Stage>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_stage_status(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStageStatusRequest>,
) -> Response {
    if let Err(e) = auth.require("olympiad.stage.manage") {
        return e.into_response();
    }
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
    responses((status = 200, description = "List of event categories", body = inline(ApiResponse<Vec<EventCategory>>)))
)]
pub async fn list_event_categories(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
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
        (status = 201, description = "Event category created", body = inline(ApiResponse<EventCategory>)),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_event_category(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(event_id): Path<Uuid>,
    Json(req): Json<CreateEventCategoryRequest>,
) -> Response {
    if let Err(e) = auth.require("olympiad.master.create") {
        return e.into_response();
    }
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
