use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exam {
    pub id: Uuid,
    pub title: String,
    pub tier: String,
    pub duration_minutes: i32,
    pub total_questions: i32,
    pub passing_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: Uuid,
    pub exam_id: Uuid,
    pub question_text: String,
    pub question_type: String,
    pub points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExamSession {
    pub id: Uuid,
    pub participant_id: Uuid,
    pub exam_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: String,
}
