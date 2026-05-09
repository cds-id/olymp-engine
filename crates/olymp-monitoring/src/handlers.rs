use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use sqlx::PgPool;
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use uuid::Uuid;

use crate::models::*;
use crate::repository::MonitoringRepository;
use olymp_core::response::{ApiResponse, WithStatus};

/// Shared state for monitoring SSE
#[derive(Clone)]
pub struct MonitoringState {
    pub pool: PgPool,
    pub tx: broadcast::Sender<MonitorEvent>,
}

// ─── Cheating Logs ───

#[utoipa::path(
    post,
    path = "/api/cheating-logs",
    tag = "monitoring",
    request_body = CreateCheatingLogRequest,
    responses(
        (status = 201, description = "Cheating log recorded"),
    )
)]
pub async fn create_cheating_log(
    State(state): State<MonitoringState>,
    Json(req): Json<CreateCheatingLogRequest>,
) -> Response {
    match MonitoringRepository::create_cheating_log(&state.pool, &req).await {
        Ok(log) => {
            // Broadcast to SSE listeners
            let _ = state.tx.send(MonitorEvent {
                event_type: "cheating".into(),
                exam_session_id: log.exam_session_id,
                data: serde_json::json!({
                    "event_type": log.event_type,
                    "detail": log.detail,
                }),
                timestamp: log.occurred_at,
            });

            // Update cheating_log_count on participant_stages
            let count =
                MonitoringRepository::count_cheating_logs(&state.pool, log.exam_session_id)
                    .await
                    .unwrap_or(0);
            let _ = sqlx::query(
                "UPDATE participant_stages SET cheating_log_count = $2, updated_at = now()
                 FROM exam_sessions es
                 WHERE participant_stages.id = es.participant_stage_id AND es.id = $1",
            )
            .bind(log.exam_session_id)
            .bind(count as i32)
            .execute(&state.pool)
            .await;

            WithStatus(StatusCode::CREATED, ApiResponse::success(log)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/sessions/{session_id}/cheating-logs",
    tag = "monitoring",
    params(("session_id" = Uuid, Path, description = "Exam session ID")),
    responses((status = 200, description = "Cheating logs for session"))
)]
pub async fn list_cheating_logs(
    State(state): State<MonitoringState>,
    Path(session_id): Path<Uuid>,
) -> Response {
    match MonitoringRepository::list_cheating_logs_by_session(&state.pool, session_id).await {
        Ok(logs) => ApiResponse::success(logs).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Exam Progress ───

#[utoipa::path(
    put,
    path = "/api/sessions/{session_id}/progress",
    tag = "monitoring",
    params(("session_id" = Uuid, Path, description = "Exam session ID")),
    request_body = UpdateProgressRequest,
    responses((status = 200, description = "Progress updated"))
)]
pub async fn update_progress(
    State(state): State<MonitoringState>,
    Path(session_id): Path<Uuid>,
    Json(req): Json<UpdateProgressRequest>,
) -> Response {
    match MonitoringRepository::upsert_progress(&state.pool, session_id, &req).await {
        Ok(progress) => {
            // Broadcast progress update
            let _ = state.tx.send(MonitorEvent {
                event_type: "progress".into(),
                exam_session_id: session_id,
                data: serde_json::json!({
                    "questions_answered": progress.questions_answered,
                    "total_questions": progress.total_questions,
                }),
                timestamp: progress.last_activity,
            });
            ApiResponse::success(progress).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/sessions/{session_id}/progress",
    tag = "monitoring",
    params(("session_id" = Uuid, Path, description = "Exam session ID")),
    responses(
        (status = 200, description = "Current progress"),
        (status = 404, description = "No progress yet")
    )
)]
pub async fn get_progress(
    State(state): State<MonitoringState>,
    Path(session_id): Path<Uuid>,
) -> Response {
    match MonitoringRepository::get_progress(&state.pool, session_id).await {
        Ok(Some(p)) => ApiResponse::success(p).into_response(),
        Ok(None) => {
            olymp_core::AppError::NotFound("No progress recorded yet".into()).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/exams/{exam_id}/progress",
    tag = "monitoring",
    params(("exam_id" = Uuid, Path, description = "Exam ID")),
    responses((status = 200, description = "All participant progress for exam"))
)]
pub async fn list_exam_progress(
    State(state): State<MonitoringState>,
    Path(exam_id): Path<Uuid>,
) -> Response {
    match MonitoringRepository::list_progress_by_exam(&state.pool, exam_id).await {
        Ok(list) => ApiResponse::success(list).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Audit Logs ───

#[utoipa::path(
    get,
    path = "/api/audit-logs",
    tag = "monitoring",
    params(
        ("actor_id" = Option<Uuid>, Query, description = "Filter by actor"),
        ("resource_type" = Option<String>, Query, description = "Filter by resource type"),
        ("resource_id" = Option<Uuid>, Query, description = "Filter by resource ID"),
        ("event_id" = Option<Uuid>, Query, description = "Filter by event"),
        ("limit" = Option<i64>, Query, description = "Max results (default 50, max 200)"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination"),
    ),
    responses((status = 200, description = "Audit logs"))
)]
pub async fn query_audit_logs(
    State(state): State<MonitoringState>,
    Query(query): Query<AuditLogQuery>,
) -> Response {
    match MonitoringRepository::query_audit_logs(&state.pool, &query).await {
        Ok(logs) => ApiResponse::success(logs).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── SSE Monitor Stream ───

#[utoipa::path(
    get,
    path = "/api/exams/{exam_id}/monitor/stream",
    tag = "monitoring",
    params(("exam_id" = Uuid, Path, description = "Exam ID to monitor")),
    responses((status = 200, description = "SSE stream of exam events"))
)]
pub async fn monitor_stream(
    State(state): State<MonitoringState>,
    Path(exam_id): Path<Uuid>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();

    // Get all session IDs for this exam to filter events
    let session_ids: Vec<Uuid> = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM exam_sessions WHERE exam_id = $1",
    )
    .bind(exam_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let stream = BroadcastStream::new(rx).filter_map(move |result| {
        match result {
            Ok(event) if session_ids.contains(&event.exam_session_id) => {
                let data = serde_json::to_string(&event).unwrap_or_default();
                Some(Ok(Event::default()
                    .event(&event.event_type)
                    .data(data)))
            }
            _ => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
