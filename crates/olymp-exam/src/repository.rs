use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use olymp_core::AppError;

pub struct ExamRepository;

impl ExamRepository {
    // ─── Exams ───

    pub async fn list_by_stage(pool: &PgPool, stage_id: Uuid) -> Result<Vec<Exam>, AppError> {
        sqlx::query_as::<_, Exam>(
            "SELECT * FROM exams WHERE stage_id = $1 ORDER BY created_at",
        )
        .bind(stage_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Exam>, AppError> {
        sqlx::query_as::<_, Exam>("SELECT * FROM exams WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn create(
        pool: &PgPool,
        stage_id: Uuid,
        req: &CreateExamRequest,
    ) -> Result<Exam, AppError> {
        sqlx::query_as::<_, Exam>(
            "INSERT INTO exams (stage_id, title, description, duration_minutes, max_attempts, shuffle_questions, shuffle_options, opens_at, closes_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *",
        )
        .bind(stage_id)
        .bind(&req.title)
        .bind(&req.description)
        .bind(req.duration_minutes)
        .bind(req.max_attempts.unwrap_or(1))
        .bind(req.shuffle_questions.unwrap_or(false))
        .bind(req.shuffle_options.unwrap_or(false))
        .bind(req.opens_at)
        .bind(req.closes_at)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateExamRequest,
    ) -> Result<Exam, AppError> {
        let current = Self::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Exam not found".into()))?;

        sqlx::query_as::<_, Exam>(
            "UPDATE exams SET title = $2, description = $3, duration_minutes = $4,
             max_attempts = $5, shuffle_questions = $6, shuffle_options = $7,
             opens_at = $8, closes_at = $9, updated_at = now()
             WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(req.title.as_deref().unwrap_or(&current.title))
        .bind(req.description.as_deref().or(current.description.as_deref()))
        .bind(req.duration_minutes.unwrap_or(current.duration_minutes))
        .bind(req.max_attempts.unwrap_or(current.max_attempts))
        .bind(req.shuffle_questions.unwrap_or(current.shuffle_questions))
        .bind(req.shuffle_options.unwrap_or(current.shuffle_options))
        .bind(req.opens_at.or(current.opens_at))
        .bind(req.closes_at.or(current.closes_at))
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Questions ───

    pub async fn list_questions(pool: &PgPool, exam_id: Uuid) -> Result<Vec<Question>, AppError> {
        sqlx::query_as::<_, Question>(
            "SELECT * FROM questions WHERE exam_id = $1 ORDER BY sequence",
        )
        .bind(exam_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn list_questions_paginated(
        pool: &PgPool,
        exam_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Question>, AppError> {
        sqlx::query_as::<_, Question>(
            "SELECT * FROM questions WHERE exam_id = $1 ORDER BY sequence LIMIT $2 OFFSET $3",
        )
        .bind(exam_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn create_question(
        pool: &PgPool,
        exam_id: Uuid,
        req: &CreateQuestionRequest,
    ) -> Result<Question, AppError> {
        // Validate question_type
        match req.question_type.as_str() {
            "multiple_choice" | "essay" | "true_false" => {}
            _ => {
                return Err(AppError::BadRequest(
                    "question_type must be: multiple_choice, essay, true_false".into(),
                ))
            }
        }

        sqlx::query_as::<_, Question>(
            "INSERT INTO questions (exam_id, question_text, question_type, options, correct_answer, points, sequence)
             VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        )
        .bind(exam_id)
        .bind(&req.question_text)
        .bind(&req.question_type)
        .bind(&req.options)
        .bind(&req.correct_answer)
        .bind(req.points)
        .bind(req.sequence)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn count_questions(pool: &PgPool, exam_id: Uuid) -> Result<i64, AppError> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM questions WHERE exam_id = $1")
            .bind(exam_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)
    }

    // ─── Sessions ───

    pub async fn get_session(pool: &PgPool, id: Uuid) -> Result<Option<ExamSession>, AppError> {
        sqlx::query_as::<_, ExamSession>("SELECT * FROM exam_sessions WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn list_sessions_by_exam(
        pool: &PgPool,
        exam_id: Uuid,
    ) -> Result<Vec<ExamSession>, AppError> {
        sqlx::query_as::<_, ExamSession>(
            "SELECT * FROM exam_sessions WHERE exam_id = $1 ORDER BY created_at",
        )
        .bind(exam_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Assign participant to exam (creates session with status='assigned')
    pub async fn assign_session(
        pool: &PgPool,
        participant_stage_id: Uuid,
        exam_id: Uuid,
    ) -> Result<ExamSession, AppError> {
        sqlx::query_as::<_, ExamSession>(
            "INSERT INTO exam_sessions (participant_stage_id, exam_id)
             VALUES ($1, $2) RETURNING *",
        )
        .bind(participant_stage_id)
        .bind(exam_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db) if db.constraint().is_some() => {
                AppError::Conflict("Session already exists for this participant/exam".into())
            }
            other => AppError::Database(other),
        })
    }

    /// Start exam session (assigned → in_progress)
    pub async fn start_session(pool: &PgPool, session_id: Uuid) -> Result<ExamSession, AppError> {
        let session = Self::get_session(pool, session_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".into()))?;

        if session.status != "assigned" {
            return Err(AppError::BadRequest(format!(
                "Cannot start session in '{}' status",
                session.status
            )));
        }

        // Check exam time window
        let exam = Self::get_by_id(pool, session.exam_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Exam not found".into()))?;

        let now = Utc::now();
        if let Some(opens) = exam.opens_at {
            if now < opens {
                return Err(AppError::BadRequest("Exam has not opened yet".into()));
            }
        }
        if let Some(closes) = exam.closes_at {
            if now > closes {
                return Err(AppError::BadRequest("Exam has closed".into()));
            }
        }

        sqlx::query_as::<_, ExamSession>(
            "UPDATE exam_sessions SET status = 'in_progress', started_at = now()
             WHERE id = $1 RETURNING *",
        )
        .bind(session_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Submit exam session — auto-score MCQ/true_false, calculate total
    pub async fn submit_session(
        pool: &PgPool,
        session_id: Uuid,
        auto: bool,
    ) -> Result<ScoreResult, AppError> {
        let session = Self::get_session(pool, session_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".into()))?;

        if session.status != "in_progress" {
            return Err(AppError::BadRequest(format!(
                "Cannot submit session in '{}' status",
                session.status
            )));
        }

        let started_at = session
            .started_at
            .ok_or_else(|| AppError::Internal("Session has no started_at".into()))?;
        let now = Utc::now();
        let completion_time_secs = (now - started_at).num_seconds() as i32;

        // Auto-score MCQ and true_false answers
        let questions = Self::list_questions(pool, session.exam_id).await?;
        let answers = Self::list_answers(pool, session_id).await?;

        let mut total_score: f64 = 0.0;
        let mut correct_count: i32 = 0;
        let max_score: f64 = questions.iter().map(|q| q.points).sum();

        for answer in &answers {
            let question = questions.iter().find(|q| q.id == answer.question_id);
            if let Some(q) = question {
                match q.question_type.as_str() {
                    "multiple_choice" | "true_false" => {
                        let is_correct = q
                            .correct_answer
                            .as_ref()
                            .map(|ca| Some(ca) == answer.answer_data.as_ref())
                            .unwrap_or(false);

                        let points = if is_correct { q.points } else { 0.0 };
                        if is_correct {
                            correct_count += 1;
                        }
                        total_score += points;

                        // Update answer record
                        sqlx::query(
                            "UPDATE answers SET is_correct = $2, points_earned = $3 WHERE id = $1",
                        )
                        .bind(answer.id)
                        .bind(is_correct)
                        .bind(points)
                        .execute(pool)
                        .await
                        .map_err(AppError::Database)?;
                    }
                    "essay" => {
                        // Essay: keep points_earned NULL until manual grading
                        if let Some(pts) = answer.points_earned {
                            total_score += pts;
                            if pts > 0.0 {
                                correct_count += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Update session
        sqlx::query(
            "UPDATE exam_sessions SET status = 'submitted', finished_at = $2,
             score = $3, completion_time_secs = $4, is_auto_submitted = $5
             WHERE id = $1",
        )
        .bind(session_id)
        .bind(now)
        .bind(total_score)
        .bind(completion_time_secs)
        .bind(auto)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        // Write score + completion_time back to participant_stages
        sqlx::query(
            "UPDATE participant_stages SET score = $2, completion_time_secs = $3, updated_at = now()
             WHERE id = $1",
        )
        .bind(session.participant_stage_id)
        .bind(total_score)
        .bind(completion_time_secs)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(ScoreResult {
            session_id,
            total_score,
            max_score,
            completion_time_secs,
            correct_count,
            total_questions: questions.len() as i32,
        })
    }

    /// Check if user owns session (through participant → participant_stage → session)
    pub async fn user_owns_session(
        pool: &PgPool,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<bool, AppError> {
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(
                SELECT 1 FROM exam_sessions es
                JOIN participant_stages ps ON ps.id = es.participant_stage_id
                JOIN participants p ON p.id = ps.participant_id
                WHERE es.id = $1 AND p.user_id = $2
            )",
        )
        .bind(session_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Answers ───

    pub async fn list_answers(
        pool: &PgPool,
        session_id: Uuid,
    ) -> Result<Vec<Answer>, AppError> {
        sqlx::query_as::<_, Answer>(
            "SELECT * FROM answers WHERE exam_session_id = $1 ORDER BY answered_at",
        )
        .bind(session_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    /// Save single answer (upsert)
    pub async fn save_answer(
        pool: &PgPool,
        session_id: Uuid,
        question_id: Uuid,
        answer_data: &serde_json::Value,
    ) -> Result<Answer, AppError> {
        // Verify session is in_progress
        let session = Self::get_session(pool, session_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".into()))?;

        if session.status != "in_progress" {
            return Err(AppError::BadRequest(
                "Can only submit answers during in_progress session".into(),
            ));
        }

        // Check time limit
        if let Some(started_at) = session.started_at {
            let exam = Self::get_by_id(pool, session.exam_id)
                .await?
                .ok_or_else(|| AppError::NotFound("Exam not found".into()))?;
            let elapsed = (Utc::now() - started_at).num_minutes();
            if elapsed > exam.duration_minutes as i64 {
                return Err(AppError::BadRequest("Exam time has expired".into()));
            }
        }

        sqlx::query_as::<_, Answer>(
            "INSERT INTO answers (exam_session_id, question_id, answer_data)
             VALUES ($1, $2, $3)
             ON CONFLICT (exam_session_id, question_id)
             DO UPDATE SET answer_data = $3, answered_at = now()
             RETURNING *",
        )
        .bind(session_id)
        .bind(question_id)
        .bind(answer_data)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }
}
