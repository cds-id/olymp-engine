use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use crate::repository::ExamRepository;
use olymp_core::response::{ApiResponse, WithStatus};

// ─── Exams ───

#[utoipa::path(
    get,
    path = "/api/stages/{stage_id}/exams",
    tag = "exams",
    params(("stage_id" = Uuid, Path, description = "Stage ID")),
    responses((status = 200, description = "List of exams for stage"))
)]
pub async fn list_exams(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
) -> Response {
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
        (status = 201, description = "Exam created"),
        (status = 400, description = "Bad request")
    )
)]
pub async fn create_exam(
    State(pool): State<PgPool>,
    Path(stage_id): Path<Uuid>,
    Json(req): Json<CreateExamRequest>,
) -> Response {
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
        (status = 200, description = "Exam detail"),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_exam(
    State(pool): State<PgPool>,
    Path(exam_id): Path<Uuid>,
) -> Response {
    match ExamRepository::get_by_id(&pool, exam_id).await {
        Ok(Some(exam)) => ApiResponse::success(exam).into_response(),
        Ok(None) => olymp_core::AppError::NotFound("Exam not found".into()).into_response(),
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
        (status = 200, description = "Exam updated"),
        (status = 404, description = "Not found")
    )
)]
pub async fn update_exam(
    State(pool): State<PgPool>,
    Path(exam_id): Path<Uuid>,
    Json(req): Json<UpdateExamRequest>,
) -> Response {
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
    params(("exam_id" = Uuid, Path, description = "Exam ID")),
    responses((status = 200, description = "List of questions (admin view with answers)"))
)]
pub async fn list_questions(
    State(pool): State<PgPool>,
    Path(exam_id): Path<Uuid>,
) -> Response {
    match ExamRepository::list_questions(&pool, exam_id).await {
        Ok(questions) => ApiResponse::success(questions).into_response(),
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
        (status = 201, description = "Question created"),
        (status = 400, description = "Invalid question type")
    )
)]
pub async fn create_question(
    State(pool): State<PgPool>,
    Path(exam_id): Path<Uuid>,
    Json(req): Json<CreateQuestionRequest>,
) -> Response {
    match ExamRepository::create_question(&pool, exam_id, &req).await {
        Ok(q) => WithStatus(StatusCode::CREATED, ApiResponse::success(q)).into_response(),
        Err(e) => e.into_response(),
    }
}

// ─── Sessions ───

#[utoipa::path(
    post,
    path = "/api/exams/{exam_id}/sessions",
    tag = "exam-sessions",
    params(("exam_id" = Uuid, Path, description = "Exam ID")),
    responses(
        (status = 201, description = "Session assigned"),
        (status = 409, description = "Already assigned")
    )
)]
pub async fn assign_session(
    State(pool): State<PgPool>,
    Path(exam_id): Path<Uuid>,
    Json(req): Json<AssignSessionRequest>,
) -> Response {
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
        (status = 200, description = "Session started, questions returned"),
        (status = 400, description = "Cannot start")
    )
)]
pub async fn start_session(
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Response {
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
        (status = 200, description = "Answer saved"),
        (status = 400, description = "Session not in progress or time expired")
    )
)]
pub async fn save_answer(
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
    Json(req): Json<SubmitAnswerRequest>,
) -> Response {
    match ExamRepository::save_answer(&pool, session_id, req.question_id, &req.answer_data).await {
        Ok(answer) => ApiResponse::success(answer).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/sessions/{session_id}/submit",
    tag = "exam-sessions",
    params(("session_id" = Uuid, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Session submitted and scored"),
        (status = 400, description = "Cannot submit")
    )
)]
pub async fn submit_session(
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Response {
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
        (status = 200, description = "Session detail"),
        (status = 404, description = "Not found")
    )
)]
pub async fn get_session(
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Response {
    match ExamRepository::get_session(&pool, session_id).await {
        Ok(Some(s)) => ApiResponse::success(s).into_response(),
        Ok(None) => olymp_core::AppError::NotFound("Session not found".into()).into_response(),
        Err(e) => e.into_response(),
    }
}

#[utoipa::path(
    get,
    path = "/api/sessions/{session_id}/answers",
    tag = "exam-sessions",
    params(("session_id" = Uuid, Path, description = "Session ID")),
    responses((status = 200, description = "List of answers"))
)]
pub async fn list_answers(
    State(pool): State<PgPool>,
    Path(session_id): Path<Uuid>,
) -> Response {
    match ExamRepository::list_answers(&pool, session_id).await {
        Ok(answers) => ApiResponse::success(answers).into_response(),
        Err(e) => e.into_response(),
    }
}
