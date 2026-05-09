use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ─── DB Models ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CertificateTemplate {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    pub template_url: String,
    pub stage_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Certificate {
    pub id: Uuid,
    pub template_id: Uuid,
    pub participant_stage_id: Uuid,
    pub certificate_url: Option<String>,
    pub certificate_number: Option<String>,
    pub generated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ─── Request DTOs ───

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub template_url: String,
    /// If set, template applies only to this stage. NULL = event-wide (participation cert).
    pub stage_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub template_url: Option<String>,
    pub stage_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct GenerateCertificatesRequest {
    /// If provided, only generate for specific participant_stage IDs.
    /// If empty/null, generate for all eligible participants in stage.
    pub participant_stage_ids: Option<Vec<Uuid>>,
}

// ─── Response DTOs ───

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct GenerationResult {
    pub stage_id: Uuid,
    pub template_id: Uuid,
    pub generated_count: i32,
    pub skipped_count: i32,
}
