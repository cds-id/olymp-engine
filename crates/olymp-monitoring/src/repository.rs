use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use olymp_core::AppError;

pub struct MonitoringRepository;

impl MonitoringRepository {
    // ─── Cheating Logs (immutable — insert + read only) ───

    pub async fn create_cheating_log(
        pool: &PgPool,
        req: &CreateCheatingLogRequest,
    ) -> Result<CheatingLog, AppError> {
        sqlx::query_as::<_, CheatingLog>(
            "INSERT INTO cheating_logs (exam_session_id, event_type, detail)
             VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(req.exam_session_id)
        .bind(&req.event_type)
        .bind(&req.detail)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn list_cheating_logs_by_session(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<Vec<CheatingLog>, AppError> {
        sqlx::query_as::<_, CheatingLog>(
            "SELECT * FROM cheating_logs WHERE exam_session_id = $1 ORDER BY occurred_at",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn count_cheating_logs(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<i64, AppError> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM cheating_logs WHERE exam_session_id = $1",
        )
        .bind(session_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Exam Progress (upsert per session) ───

    pub async fn upsert_progress(
        pool: &PgPool,
        session_id: Uuid,
        req: &UpdateProgressRequest,
    ) -> Result<ExamProgress, AppError> {
        sqlx::query_as::<_, ExamProgress>(
            "INSERT INTO exam_progress (exam_session_id, questions_answered, total_questions)
             VALUES ($1, $2, $3)
             ON CONFLICT (exam_session_id)
             DO UPDATE SET questions_answered = $2, total_questions = $3, last_activity = now()
             RETURNING *",
        )
        .bind(session_id)
        .bind(req.questions_answered)
        .bind(req.total_questions)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn get_progress(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<Option<ExamProgress>, AppError> {
        sqlx::query_as::<_, ExamProgress>(
            "SELECT * FROM exam_progress WHERE exam_session_id = $1",
        )
        .bind(session_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Bulk progress for all sessions in an exam (for monitor dashboard)
    pub async fn list_progress_by_exam(
        pool: &PgPool,
        exam_id: Uuid,
    ) -> Result<Vec<ExamProgress>, AppError> {
        sqlx::query_as::<_, ExamProgress>(
            "SELECT ep.* FROM exam_progress ep
             JOIN exam_sessions es ON es.id = ep.exam_session_id
             WHERE es.exam_id = $1
             ORDER BY ep.last_activity DESC",
        )
        .bind(exam_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Audit Logs (immutable — insert + read only) ───

    pub async fn create_audit_log(
        pool: &PgPool,
        actor_id: Option<Uuid>,
        req: &CreateAuditLogRequest,
        ip_address: Option<&str>,
    ) -> Result<AuditLog, AppError> {
        sqlx::query_as::<_, AuditLog>(
            "INSERT INTO audit_logs (actor_id, action, resource_type, resource_id, event_id, metadata, ip_address)
             VALUES ($1, $2, $3, $4, $5, $6, $7::INET) RETURNING *",
        )
        .bind(actor_id)
        .bind(&req.action)
        .bind(&req.resource_type)
        .bind(req.resource_id)
        .bind(req.event_id)
        .bind(&req.metadata)
        .bind(ip_address)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn query_audit_logs(
        pool: &PgPool,
        query: &AuditLogQuery,
    ) -> Result<Vec<AuditLog>, AppError> {
        let limit = query.limit.unwrap_or(50).min(200);
        let offset = query.offset.unwrap_or(0);

        // Build dynamic query with optional filters
        let mut sql = String::from("SELECT * FROM audit_logs WHERE 1=1");
        let mut param_idx = 1u32;

        if query.actor_id.is_some() {
            sql.push_str(&format!(" AND actor_id = ${param_idx}"));
            param_idx += 1;
        }
        if query.resource_type.is_some() {
            sql.push_str(&format!(" AND resource_type = ${param_idx}"));
            param_idx += 1;
        }
        if query.resource_id.is_some() {
            sql.push_str(&format!(" AND resource_id = ${param_idx}"));
            param_idx += 1;
        }
        if query.event_id.is_some() {
            sql.push_str(&format!(" AND event_id = ${param_idx}"));
            param_idx += 1;
        }

        sql.push_str(&format!(
            " ORDER BY created_at DESC LIMIT ${param_idx} OFFSET ${}",
            param_idx + 1
        ));

        let mut q = sqlx::query_as::<_, AuditLog>(&sql);

        if let Some(v) = query.actor_id {
            q = q.bind(v);
        }
        if let Some(ref v) = query.resource_type {
            q = q.bind(v);
        }
        if let Some(v) = query.resource_id {
            q = q.bind(v);
        }
        if let Some(v) = query.event_id {
            q = q.bind(v);
        }

        q = q.bind(limit).bind(offset);

        q.fetch_all(pool).await.map_err(AppError::Database)
    }
}
