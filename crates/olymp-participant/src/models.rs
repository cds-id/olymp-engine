use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum Tier {
    #[serde(rename = "district")]
    District,
    #[serde(rename = "province")]
    Province,
    #[serde(rename = "national")]
    National,
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tier::District => write!(f, "district"),
            Tier::Province => write!(f, "province"),
            Tier::National => write!(f, "national"),
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub current_tier: String,
    pub is_locked: bool,
    pub locked_by_account: Option<String>,
    pub score: f64,
    pub rank: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TierProgression {
    pub id: Uuid,
    pub participant_id: Uuid,
    pub from_tier: String,
    pub to_tier: String,
    pub advanced_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterParticipantRequest {
    pub tier: String,
    pub school: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParticipantResponse {
    pub id: Uuid,
    pub current_tier: String,
    pub is_locked: bool,
    pub score: f64,
    pub rank: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdvanceTierRequest {
    pub to_tier: String,
}
