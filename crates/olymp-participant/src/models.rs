use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ─── DB Models ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Participant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub education_level_id: Uuid,
    pub subject_id: Uuid,
    pub school_name: Option<String>,
    pub district_id: Option<Uuid>,
    pub province_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ParticipantStage {
    pub id: Uuid,
    pub participant_id: Uuid,
    pub stage_id: Uuid,
    pub status: String,
    pub score: Option<f64>,
    pub completion_time_secs: Option<i32>,
    pub rank: Option<i32>,
    pub cheating_log_count: i32,
    pub promoted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Request DTOs ───

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RegisterParticipantRequest {
    pub user_id: Uuid,
    pub education_level_id: Uuid,
    pub subject_id: Uuid,
    pub school_name: Option<String>,
    pub district_id: Option<Uuid>,
    pub province_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateParticipantRequest {
    pub school_name: Option<String>,
    pub district_id: Option<Uuid>,
    pub province_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct VerifyRejectRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BatchParticipantRequest {
    /// If empty/null, applies to ALL eligible participants in stage
    pub participant_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BatchResult {
    pub affected: i32,
    pub skipped: i32,
    pub errors: Vec<String>,
}

// ─── Response DTOs ───

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ParticipantDetail {
    #[serde(flatten)]
    pub participant: Participant,
    pub stages: Vec<ParticipantStage>,
}

#[derive(Debug, FromRow, Serialize, utoipa::ToSchema)]
pub struct ParticipantListItem {
    pub id: Uuid,
    pub user_id: Uuid,
    pub school_name: Option<String>,
    pub district_id: Option<Uuid>,
    pub province_id: Option<Uuid>,
    pub stage_status: Option<String>,
    pub score: Option<f64>,
    pub rank: Option<i32>,
}

/// Valid status transitions for participant stages
impl ParticipantStage {
    pub fn valid_transitions(status: &str) -> &'static [&'static str] {
        match status {
            "registered" => &["verified", "disqualified"],
            "verified" => &["assigned_to_exam", "disqualified"],
            "assigned_to_exam" => &["in_progress", "disqualified"],
            "in_progress" => &["submitted", "disqualified"],
            "submitted" => &["scored", "disqualified"],
            "scored" => &["ranked", "disqualified"],
            "ranked" => &["qualified", "not_qualified", "winner", "finalist", "disqualified"],
            "qualified" => &["disqualified"],
            _ => &[],
        }
    }

    pub fn can_transition_to(&self, new_status: &str) -> bool {
        Self::valid_transitions(&self.status).contains(&new_status)
    }
}
