use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use olymp_core::types::{EventStatus, StageStatus, Tier};

// ─── Education Level ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EducationLevel {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateEducationLevelRequest {
    pub name: String,
}

// ─── Subject ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Subject {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSubjectRequest {
    pub name: String,
}

// ─── Event ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub academic_year: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateEventRequest {
    pub name: String,
    pub academic_year: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateEventRequest {
    pub name: Option<String>,
    pub academic_year: Option<String>,
    pub status: Option<EventStatus>,
}

// ─── Stage ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Stage {
    pub id: Uuid,
    pub event_id: Uuid,
    pub tier: String,
    pub sequence: i32,
    pub status: String,
    pub registration_opens_at: Option<DateTime<Utc>>,
    pub registration_closes_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateStageRequest {
    pub tier: Tier,
    pub registration_opens_at: Option<DateTime<Utc>>,
    pub registration_closes_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateStageStatusRequest {
    pub status: StageStatus,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateStageRequest {
    pub registration_opens_at: Option<DateTime<Utc>>,
    pub registration_closes_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

// ─── Event Category ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct EventCategory {
    pub id: Uuid,
    pub event_id: Uuid,
    pub education_level_id: Uuid,
    pub subject_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateEventCategoryRequest {
    pub education_level_id: Uuid,
    pub subject_id: Uuid,
}
