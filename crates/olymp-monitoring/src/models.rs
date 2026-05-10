use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ─── DB Models ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CheatingLog {
    pub id: Uuid,
    pub exam_session_id: Uuid,
    pub event_type: String,
    pub detail: Option<serde_json::Value>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ExamProgress {
    pub id: Uuid,
    pub exam_session_id: Uuid,
    pub questions_answered: i32,
    pub total_questions: i32,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AuditLog {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ─── Request DTOs ───

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateCheatingLogRequest {
    pub exam_session_id: Uuid,
    pub event_type: String,
    pub detail: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateProgressRequest {
    pub questions_answered: i32,
    pub total_questions: i32,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateAuditLogRequest {
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AuditLogQuery {
    pub actor_id: Option<Uuid>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    /// Page number (1-based, default: 1)
    pub page: Option<u32>,
    /// Items per page (default: 20, max: 100)
    pub per_page: Option<u32>,
}

// ─── SSE Event ───

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorEvent {
    pub event_type: String,
    pub exam_session_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}
