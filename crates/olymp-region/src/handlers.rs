use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::repository::RegionRepository;
use olymp_core::response::{ApiResponse, WithStatus};
use olymp_core::AppError;

// ─── Province handlers ───

#[utoipa::path(
    get,
    path = "/api/provinces",
    tag = "regions",
    responses(
        (status = 200, description = "List of provinces", body = Vec<Province>)
    )
)]
pub async fn list_provinces(State(pool): State<PgPool>) -> Response {
    match RegionRepository::list_provinces(&pool).await {
        Ok(provinces) => ApiResponse::success(provinces).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/provinces/{id}",
    tag = "regions",
    params(("id" = Uuid, Path, description = "Province ID")),
    responses(
        (status = 200, description = "Province details", body = Province),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_province(State(pool): State<PgPool>, Path(id): Path<Uuid>) -> Response {
    match RegionRepository::get_province(&pool, id).await {
        Ok(Some(p)) => ApiResponse::success(p).into_response(),
        Ok(None) => AppError::NotFound("Province not found".into()).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/provinces",
    tag = "regions",
    request_body = CreateProvinceRequest,
    responses(
        (status = 201, description = "Province created", body = Province),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_province(
    State(pool): State<PgPool>,
    Json(req): Json<CreateProvinceRequest>,
) -> Response {
    match RegionRepository::create_province(&pool, &req.name).await {
        Ok(p) => WithStatus(StatusCode::CREATED, ApiResponse::success(p)).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── District handlers ───

#[utoipa::path(
    get,
    path = "/api/provinces/{province_id}/districts",
    tag = "regions",
    params(("province_id" = Uuid, Path, description = "Province ID")),
    responses(
        (status = 200, description = "List of districts in province", body = Vec<District>)
    )
)]
pub async fn list_districts(
    State(pool): State<PgPool>,
    Path(province_id): Path<Uuid>,
) -> Response {
    match RegionRepository::list_districts(&pool, province_id).await {
        Ok(districts) => ApiResponse::success(districts).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/provinces/{province_id}/districts",
    tag = "regions",
    params(("province_id" = Uuid, Path, description = "Province ID")),
    request_body = CreateDistrictRequest,
    responses(
        (status = 201, description = "District created", body = District),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_district(
    State(pool): State<PgPool>,
    Path(province_id): Path<Uuid>,
    Json(req): Json<CreateDistrictRequest>,
) -> Response {
    match RegionRepository::create_district(&pool, province_id, &req.name).await {
        Ok(d) => WithStatus(StatusCode::CREATED, ApiResponse::success(d)).into_response(),
        Err(e) => e.into_response(),
    }
}
