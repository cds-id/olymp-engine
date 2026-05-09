use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub actor_id: Option<Uuid>,
    pub changes: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamProgress {
    pub id: Uuid,
    pub exam_session_id: Uuid,
    pub participant_id: Uuid,
    pub questions_answered: i32,
    pub total_questions: i32,
    pub last_activity: DateTime<Utc>,
}
