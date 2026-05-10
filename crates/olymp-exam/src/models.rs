use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ─── DB Models ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Exam {
    pub id: Uuid,
    pub stage_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub max_attempts: i32,
    pub shuffle_questions: bool,
    pub shuffle_options: bool,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Question {
    pub id: Uuid,
    pub exam_id: Uuid,
    pub question_text: String,
    pub question_type: String,
    pub options: Option<serde_json::Value>,
    pub correct_answer: Option<serde_json::Value>,
    pub points: f64,
    pub sequence: i32,
    pub created_at: DateTime<Utc>,
}

/// Question without correct_answer — served to participants during exam
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct QuestionForParticipant {
    pub id: Uuid,
    pub question_text: String,
    pub question_type: String,
    pub options: Option<serde_json::Value>,
    pub points: f64,
    pub sequence: i32,
}

impl From<Question> for QuestionForParticipant {
    fn from(q: Question) -> Self {
        Self {
            id: q.id,
            question_text: q.question_text,
            question_type: q.question_type,
            options: q.options,
            points: q.points,
            sequence: q.sequence,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ExamSession {
    pub id: Uuid,
    pub participant_stage_id: Uuid,
    pub exam_id: Uuid,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub score: Option<f64>,
    pub completion_time_secs: Option<i32>,
    pub status: String,
    pub is_auto_submitted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Answer {
    pub id: Uuid,
    pub exam_session_id: Uuid,
    pub question_id: Uuid,
    pub answer_data: Option<serde_json::Value>,
    pub is_correct: Option<bool>,
    pub points_earned: Option<f64>,
    pub answered_at: DateTime<Utc>,
}

/// Answer without grading info — served to participants during active sessions
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct AnswerForParticipant {
    pub id: Uuid,
    pub exam_session_id: Uuid,
    pub question_id: Uuid,
    pub answer_data: Option<serde_json::Value>,
    pub answered_at: DateTime<Utc>,
}

impl From<Answer> for AnswerForParticipant {
    fn from(a: Answer) -> Self {
        Self {
            id: a.id,
            exam_session_id: a.exam_session_id,
            question_id: a.question_id,
            answer_data: a.answer_data,
            answered_at: a.answered_at,
        }
    }
}

// ─── Request DTOs ───

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateExamRequest {
    pub title: String,
    pub description: Option<String>,
    pub duration_minutes: i32,
    pub max_attempts: Option<i32>,
    pub shuffle_questions: Option<bool>,
    pub shuffle_options: Option<bool>,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateExamRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub duration_minutes: Option<i32>,
    pub max_attempts: Option<i32>,
    pub shuffle_questions: Option<bool>,
    pub shuffle_options: Option<bool>,
    pub opens_at: Option<DateTime<Utc>>,
    pub closes_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateQuestionRequest {
    pub question_text: String,
    pub question_type: String,
    pub options: Option<serde_json::Value>,
    pub correct_answer: Option<serde_json::Value>,
    pub points: f64,
    pub sequence: i32,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AssignSessionRequest {
    pub participant_stage_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SubmitAnswerRequest {
    pub question_id: Uuid,
    pub answer_data: serde_json::Value,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct BatchSubmitAnswersRequest {
    pub answers: Vec<SubmitAnswerRequest>,
}

// ─── Response DTOs ───

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ExamWithQuestionCount {
    #[serde(flatten)]
    pub exam: Exam,
    pub question_count: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SessionStartResponse {
    pub session: ExamSession,
    pub questions: Vec<QuestionForParticipant>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ScoreResult {
    pub session_id: Uuid,
    pub total_score: f64,
    pub max_score: f64,
    pub completion_time_secs: i32,
    pub correct_count: i32,
    pub total_questions: i32,
}
