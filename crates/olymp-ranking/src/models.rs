use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub id: Uuid,
    pub participant_id: Uuid,
    pub exam_id: Uuid,
    pub score: f64,
    pub graded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leaderboard {
    pub id: Uuid,
    pub tier: String,
    pub participant_id: Uuid,
    pub rank: i32,
    pub score: f64,
    pub updated_at: DateTime<Utc>,
}
