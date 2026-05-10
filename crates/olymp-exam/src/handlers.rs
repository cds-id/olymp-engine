use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::repository::ExamRepository;
use olymp_core::auth::AuthContext;
use olymp_core::response::{ApiResponse, Meta, WithStatus};
use olymp_core::types::ListParams;
use olymp_core::AppError;

// ─── My Sessions (peserta self-service) ───

#[utoipa::path(
    get,
    path = "/api/users/me/sessions",
    tag = "exam-sessions",
    responses(
        (status = 200, description = "Current user's exam sessions", body = inline(ApiResponse<Vec<ExamSession>>)),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer" = []))
)]
pub async fn my_sessions(
    auth: AuthContext,
    State(pool): State<PgPool>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    match ExamRepository::list_sessions_by_user(&pool, auth.user_id).await {
        Ok(sessions) => ApiResponse::success(sessions).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Exams ───

#[utoipa::path(
    get,
    path = "/api/stages/{stage_id}/exams",
    tag = "exams",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    responses((status = 200, description = "List of exams for stage", body = inline(ApiResponse<Vec<Exam>>)))
)]
pub async fn list_exams(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    match ExamRepository::list_by_stage(&pool, stage_id).await {
        Ok(exams) => ApiResponse::success(exams).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/stages/{stage_id}/exams",
    tag = "exams",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    request_body = CreateExamRequest,
    responses(
        (status = 201, description = "Exam created", body = inline(ApiResponse<Exam>)),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_exam(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Json(req): Json<CreateExamRequest>,
) -> Response {
    if let Err(e) = auth.require("exam.create") {
        return e.into_response();
    }
    match ExamRepository::create(&pool, stage_id, &req).await {
        Ok(exam) => WithStatus(StatusCode::CREATED, ApiResponse::success(exam)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/exams/{exam_id}",
    tag = "exams",
    params(("exam_id" = Uuid, Path, description = "Exam ID")),
    responses(
        (status = 200, description = "Exam detail", body = inline(ApiResponse<Exam>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_exam(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(exam_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    match ExamRepository::get_by_id(&pool, exam_id).await {
        Ok(Some(exam)) => ApiResponse::success(exam).into_response(),
        Ok(None) => AppError::NotFound("Exam not found".into()).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/exams/{exam_id}",
    tag = "exams",
    params(("exam_id" = Uuid, Path, description = "Exam ID")),
    request_body = UpdateExamRequest,
    responses(
        (status = 200, description = "Exam updated", body = inline(ApiResponse<Exam>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_exam(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(exam_id): Path<Uuid>,
    Json(req): Json<UpdateExamRequest>,
) -> Response {
    if let Err(e) = auth.require("exam.update") {
        return e.into_response();
    }
    match ExamRepository::update(&pool, exam_id, &req).await {
        Ok(exam) => ApiResponse::success(exam).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Questions ───

#[utoipa::path(
    get,
    path = "/api/exams/{exam_id}/questions",
    tag = "exams",
    params(
        ("exam_id" = Uuid, Path, description = "Exam ID"),
        ListParams,
    ),
    responses((status = 200, description = "Paginated questions (staff sees correct_answer, participants do not)", body = inline(ApiResponse<Vec<Question>>)))
)]
pub async fn list_questions(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(exam_id): Path<Uuid>,
    Query(params): Query<ListParams>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    let total = match ExamRepository::count_questions(&pool, exam_id).await {
        Ok(t) => t,
        Err(e) => return e.into_response(),
    };
    let meta = Meta::paginated(params.page(), params.per_page(), total as u64);
    match ExamRepository::list_questions_paginated(&pool, exam_id, params.limit(), params.offset())
        .await
    {
        Ok(questions) => {
            // Staff sees full Question with correct_answer; peserta sees stripped version
            if auth.is_staff() {
                ApiResponse::success(questions)
                    .with_meta(meta)
                    .into_response()
            } else {
                let stripped: Vec<QuestionForParticipant> =
                    questions.into_iter().map(Into::into).collect();
                ApiResponse::success(stripped)
                    .with_meta(meta)
                    .into_response()
            }
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/exams/{exam_id}/questions",
    tag = "exams",
    params(("exam_id" = Uuid, Path, description = "Exam ID")),
    request_body = CreateQuestionRequest,
    responses(
        (status = 201, description = "Question created", body = inline(ApiResponse<Question>)),
        (status = 400, description = "Invalid question type")
    )
)]
pub async fn create_question(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(exam_id): Path<Uuid>,
    Json(req): Json<CreateQuestionRequest>,
) -> Response {
    if let Err(e) = auth.require("exam.create") {
        return e.into_response();
    }
    match ExamRepository::create_question(&pool, exam_id, &req).await {
        Ok(q) => WithStatus(StatusCode::CREATED, ApiResponse::success(q)).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    put,
    path = "/api/exams/{exam_id}/questions/{question_id}",
    tag = "exams",
    params(
        ("exam_id" = Uuid, Path, description = "Exam ID"),
        ("question_id" = Uuid, Path, description = "Question ID"),
    ),
    request_body = UpdateQuestionRequest,
    responses(
        (status = 200, description = "Question updated", body = inline(ApiResponse<Question>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_question(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path((_exam_id, question_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateQuestionRequest>,
) -> Response {
    if let Err(e) = auth.require("exam.update") {
        return e.into_response();
    }
    match ExamRepository::update_question(&pool, question_id, &req).await {
        Ok(q) => ApiResponse::success(q).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    delete,
    path = "/api/exams/{exam_id}/questions/{question_id}",
    tag = "exams",
    params(
        ("exam_id" = Uuid, Path, description = "Exam ID"),
        ("question_id" = Uuid, Path, description = "Question ID"),
    ),
    responses(
        (status = 200, description = "Question deleted"),
        (status = 404, description = "Not found")
    )
)]
pub async fn delete_question(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path((_exam_id, question_id)): Path<(Uuid, Uuid)>,
) -> Response {
    if let Err(e) = auth.require("exam.update") {
        return e.into_response();
    }
    match ExamRepository::delete_question(&pool, question_id).await {
        Ok(()) => ApiResponse::success(
            serde_json::json!({"message": "Question deleted"}),
        )
        .into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Sessions ───

/// Helper: verify peserta owns session, staff can access any
async fn require_session_access(
    auth: &AuthContext,
    pool: &PgPool,
    session_id: Uuid,
) -> Result<(), Response> {
    if !auth.is_staff() {
        match ExamRepository::user_owns_session(pool, auth.user_id, session_id).await {
            Ok(true) => {}
            Ok(false) => {
                return Err(
                    AppError::Forbidden("Cannot access another user's session".into())
                        .into_response(),
                );
            }
            Err(e) => return Err(e.into_response()),
        }
    }
    Ok(())
}

#[utoipa::path(
    post,
    path = "/api/exams/{exam_id}/sessions",
    tag = "exam-sessions",
    params(("exam_id" = Uuid, Path, description = "Exam ID")),
    responses(
        (status = 201, description = "Session assigned", body = inline(ApiResponse<ExamSession>)),
        (status = 409, description = "Already assigned")
    )
)]
pub async fn assign_session(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(exam_id): Path<Uuid>,
    Json(req): Json<AssignSessionRequest>,
) -> Response {
    if let Err(e) = auth.require("exam.assign") {
        return e.into_response();
    }
    match ExamRepository::assign_session(&pool, req.participant_stage_id, exam_id).await {
        Ok(session) => {
            WithStatus(StatusCode::CREATED, ApiResponse::success(session)).into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/sessions/{session_id}/start",
    tag = "exam-sessions",
    params(("session_id" = Uuid, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Session started, questions returned", body = inline(ApiResponse<SessionStartResponse>)),
        (status = 400, description = "Cannot start")
    )
)]
pub async fn start_session(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    if let Err(resp) = require_session_access(&auth, &pool, session_id).await {
        return resp;
    }

    let session = match ExamRepository::start_session(&pool, session_id).await {
        Ok(s) => s,
        Err(e) => return e.into_response(),
    };

    // Return questions without correct answers
    let questions = match ExamRepository::list_questions(&pool, session.exam_id).await {
        Ok(qs) => qs.into_iter().map(QuestionForParticipant::from).collect(),
        Err(e) => return e.into_response(),
    };

    ApiResponse::success(SessionStartResponse { session, questions }).into_response()
}

#[utoipa::path(
    post,
    path = "/api/sessions/{session_id}/answers",
    tag = "exam-sessions",
    params(("session_id" = Uuid, Path, description = "Session ID")),
    request_body = SubmitAnswerRequest,
    responses(
        (status = 200, description = "Answer saved", body = inline(ApiResponse<Answer>)),
        (status = 400, description = "Session not in progress or time expired")
    )
)]
pub async fn save_answer(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
    Json(req): Json<SubmitAnswerRequest>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    if let Err(resp) = require_session_access(&auth, &pool, session_id).await {
        return resp;
    }
    match ExamRepository::save_answer(&pool, session_id, req.question_id, &req.answer_data).await {
        Ok(answer) => {
            if auth.is_staff() {
                ApiResponse::success(answer).into_response()
            } else {
                ApiResponse::success(AnswerForParticipant::from(answer)).into_response()
            }
        }
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/sessions/{session_id}/submit",
    tag = "exam-sessions",
    params(("session_id" = Uuid, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Session submitted and scored", body = inline(ApiResponse<ScoreResult>)),
        (status = 400, description = "Cannot submit")
    )
)]
pub async fn submit_session(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    if let Err(resp) = require_session_access(&auth, &pool, session_id).await {
        return resp;
    }
    match ExamRepository::submit_session(&pool, session_id, false).await {
        Ok(result) => ApiResponse::success(result).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/sessions/{session_id}",
    tag = "exam-sessions",
    params(("session_id" = Uuid, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Session detail", body = inline(ApiResponse<ExamSession>)),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_session(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    if let Err(resp) = require_session_access(&auth, &pool, session_id).await {
        return resp;
    }
    match ExamRepository::get_session(&pool, session_id).await {
        Ok(Some(s)) => ApiResponse::success(s).into_response(),
        Ok(None) => AppError::NotFound("Session not found".into()).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/sessions/{session_id}/answers",
    tag = "exam-sessions",
    params(("session_id" = Uuid, Path, description = "Session ID")),
    responses((status = 200, description = "List of answers", body = inline(ApiResponse<Vec<Answer>>)))
)]
pub async fn list_answers(
    auth: AuthContext,
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Response {
    if let Err(e) = auth.require("exam.view") {
        return e.into_response();
    }
    if let Err(resp) = require_session_access(&auth, &pool, session_id).await {
        return resp;
    }
    match ExamRepository::list_answers(&pool, session_id).await {
        Ok(answers) => {
            // Staff sees full Answer (is_correct, points_earned); peserta gets stripped
            if auth.is_staff() {
                ApiResponse::success(answers).into_response()
            } else {
                let stripped: Vec<AnswerForParticipant> =
                    answers.into_iter().map(Into::into).collect();
                ApiResponse::success(stripped).into_response()
            }
        }
        Err(e) => e.into_response(),
    }
}
