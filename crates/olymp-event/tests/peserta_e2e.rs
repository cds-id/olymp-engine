//! End-to-end peserta flow: register → verify → assign exam → take exam → submit → score
//! Proves full competition pipeline from Daerah through to exam submission.

mod helpers;

use helpers::TestDb;
use olymp_event::repository::EventRepository;
use olymp_event::models::*;
use olymp_participant::repository::ParticipantRepository;
use olymp_participant::models::*;
use olymp_exam::repository::ExamRepository;
use olymp_exam::models::*;
use olymp_core::types::{Tier, StageStatus, EventStatus};

/// Full peserta lifecycle: register for OSN → take district exam → get scored
#[tokio::test]
async fn peserta_full_district_exam_flow() {
    let db = TestDb::new().await;

    // ── Setup competition ──
    let level = EventRepository::create_education_level(&db.pool, "SMA E2E")
        .await
        .unwrap();
    let subject = EventRepository::create_subject(&db.pool, "Matematika E2E")
        .await
        .unwrap();
    let event = EventRepository::create_event(&db.pool, "OSN E2E Test", "2025/2026")
        .await
        .unwrap();

    // 3-tier stages
    let district = EventRepository::create_stage(&db.pool, event.id, Tier::District)
        .await
        .unwrap();
    let _province = EventRepository::create_stage(&db.pool, event.id, Tier::Province)
        .await
        .unwrap();
    let _national = EventRepository::create_stage(&db.pool, event.id, Tier::National)
        .await
        .unwrap();

    // Link category
    EventRepository::create_event_category(&db.pool, event.id, level.id, subject.id)
        .await
        .unwrap();

    // Activate event
    EventRepository::update_event(
        &db.pool,
        event.id,
        &UpdateEventRequest {
            name: None,
            academic_year: None,
            status: Some(EventStatus::Active),
        },
    )
    .await
    .unwrap();

    // Open district registration
    EventRepository::update_stage_status(&db.pool, district.id, StageStatus::OpenRegistration)
        .await
        .unwrap();

    // ── Peserta registers ──
    let user_id = db.create_test_user("peserta-e2e").await;

    let participant = ParticipantRepository::register(
        &db.pool,
        event.id,
        &RegisterParticipantRequest {
            user_id,
            education_level_id: level.id,
            subject_id: subject.id,
            school_name: Some("SMAN 1 Jakarta".into()),
            district_id: None,
            province_id: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(participant.event_id, event.id);
    assert_eq!(participant.school_name.as_deref(), Some("SMAN 1 Jakarta"));

    // Create participant_stage entry for district
    let ps = ParticipantRepository::create_stage_entry(&db.pool, participant.id, district.id)
        .await
        .unwrap();
    assert_eq!(ps.status, "registered");
    assert_eq!(ps.score, None);

    // ── Admin verifies participant ──
    let ps = ParticipantRepository::transition_stage_status(&db.pool, ps.id, "verified")
        .await
        .unwrap();
    assert_eq!(ps.status, "verified");

    // ── Close registration, setup exam ──
    EventRepository::update_stage_status(&db.pool, district.id, StageStatus::RegistrationClosed)
        .await
        .unwrap();
    EventRepository::update_stage_status(&db.pool, district.id, StageStatus::Verification)
        .await
        .unwrap();
    EventRepository::update_stage_status(&db.pool, district.id, StageStatus::ReadyForExam)
        .await
        .unwrap();

    // ── Create exam with MCQ questions ──
    let exam = ExamRepository::create(
        &db.pool,
        district.id,
        &CreateExamRequest {
            title: "OSN Matematika Daerah 2026".into(),
            description: Some("Ujian tingkat kabupaten/kota".into()),
            duration_minutes: 120,
            max_attempts: None,
            shuffle_questions: Some(false),
            shuffle_options: Some(false),
            opens_at: None,
            closes_at: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(exam.title, "OSN Matematika Daerah 2026");
    assert_eq!(exam.duration_minutes, 120);

    // Add 3 MCQ questions
    let q1 = ExamRepository::create_question(
        &db.pool,
        exam.id,
        &CreateQuestionRequest {
            question_text: "Berapakah 2 + 2?".into(),
            question_type: "multiple_choice".into(),
            options: Some(serde_json::json!(["2", "3", "4", "5"])),
            correct_answer: Some(serde_json::json!("4")),
            points: 10.0,
            sequence: 1,
        },
    )
    .await
    .unwrap();

    let q2 = ExamRepository::create_question(
        &db.pool,
        exam.id,
        &CreateQuestionRequest {
            question_text: "Turunan dari x^2 adalah?".into(),
            question_type: "multiple_choice".into(),
            options: Some(serde_json::json!(["x", "2x", "x^2", "2"])),
            correct_answer: Some(serde_json::json!("2x")),
            points: 10.0,
            sequence: 2,
        },
    )
    .await
    .unwrap();

    let q3 = ExamRepository::create_question(
        &db.pool,
        exam.id,
        &CreateQuestionRequest {
            question_text: "Pi adalah bilangan irasional?".into(),
            question_type: "true_false".into(),
            options: Some(serde_json::json!(["true", "false"])),
            correct_answer: Some(serde_json::json!("true")),
            points: 5.0,
            sequence: 3,
        },
    )
    .await
    .unwrap();

    let q_count = ExamRepository::count_questions(&db.pool, exam.id).await.unwrap();
    assert_eq!(q_count, 3);

    // ── Open exam ──
    EventRepository::update_stage_status(&db.pool, district.id, StageStatus::ExamOpen)
        .await
        .unwrap();

    // ── Assign peserta to exam ──
    let ps = ParticipantRepository::transition_stage_status(&db.pool, ps.id, "assigned_to_exam")
        .await
        .unwrap();
    assert_eq!(ps.status, "assigned_to_exam");

    let session = ExamRepository::assign_session(&db.pool, ps.id, exam.id)
        .await
        .unwrap();
    assert_eq!(session.status, "assigned");

    // ── Peserta starts exam ──
    let ps = ParticipantRepository::transition_stage_status(&db.pool, ps.id, "in_progress")
        .await
        .unwrap();
    assert_eq!(ps.status, "in_progress");

    let session = ExamRepository::start_session(&db.pool, session.id)
        .await
        .unwrap();
    assert_eq!(session.status, "in_progress");
    assert!(session.started_at.is_some());

    // ── Peserta answers questions ──
    // Q1: correct (4)
    let a1 = ExamRepository::save_answer(
        &db.pool,
        session.id,
        q1.id,
        &serde_json::json!("4"),
    )
    .await
    .unwrap();
    assert_eq!(a1.question_id, q1.id);

    // Q2: wrong (should be "2x", answers "x")
    ExamRepository::save_answer(&db.pool, session.id, q2.id, &serde_json::json!("x"))
        .await
        .unwrap();

    // Q3: correct (true)
    ExamRepository::save_answer(&db.pool, session.id, q3.id, &serde_json::json!("true"))
        .await
        .unwrap();

    // Verify all 3 answers saved
    let answers = ExamRepository::list_answers(&db.pool, session.id).await.unwrap();
    assert_eq!(answers.len(), 3);

    // ── Peserta submits exam ──
    let score_result = ExamRepository::submit_session(&db.pool, session.id, false)
        .await
        .unwrap();

    // Q1 correct (10pts) + Q2 wrong (0pts) + Q3 correct (5pts) = 15
    assert_eq!(score_result.total_score, 15.0);
    assert_eq!(score_result.max_score, 25.0);
    assert_eq!(score_result.correct_count, 2);
    assert_eq!(score_result.total_questions, 3);
    assert!(score_result.completion_time_secs >= 0);

    // ── Verify score written to participant_stage ──
    let ps = ParticipantRepository::transition_stage_status(&db.pool, ps.id, "submitted")
        .await
        .unwrap();
    assert_eq!(ps.status, "submitted");
    assert_eq!(ps.score, Some(15.0));
    assert!(ps.completion_time_secs.is_some());

    // ── Transition to scored ──
    let ps = ParticipantRepository::transition_stage_status(&db.pool, ps.id, "scored")
        .await
        .unwrap();
    assert_eq!(ps.status, "scored");

    // ── Verify participant detail shows stages ──
    let stages = ParticipantRepository::get_participant_stages(&db.pool, participant.id)
        .await
        .unwrap();
    assert_eq!(stages.len(), 1); // only district so far
    assert_eq!(stages[0].status, "scored");
    assert_eq!(stages[0].score, Some(15.0));

    // ── Verify exam sessions ──
    let sessions = ExamRepository::list_sessions_by_exam(&db.pool, exam.id)
        .await
        .unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].status, "submitted");
    assert_eq!(sessions[0].score, Some(15.0));

    // ── Cleanup ──
    cleanup_full(&db, event.id, level.id, subject.id, user_id).await;
}

/// Cannot re-register for same event+subject (UNIQUE constraint)
#[tokio::test]
async fn peserta_cannot_double_register() {
    let db = TestDb::new().await;

    let level = EventRepository::create_education_level(&db.pool, "SMA Dupe")
        .await
        .unwrap();
    let subject = EventRepository::create_subject(&db.pool, "Math Dupe")
        .await
        .unwrap();
    let event = EventRepository::create_event(&db.pool, "Dupe Test", "2025/2026")
        .await
        .unwrap();
    let user_id = db.create_test_user("dupe-peserta").await;

    // First registration — OK
    ParticipantRepository::register(
        &db.pool,
        event.id,
        &RegisterParticipantRequest {
            user_id,
            education_level_id: level.id,
            subject_id: subject.id,
            school_name: None,
            district_id: None,
            province_id: None,
        },
    )
    .await
    .unwrap();

    // Second registration — should fail
    let result = ParticipantRepository::register(
        &db.pool,
        event.id,
        &RegisterParticipantRequest {
            user_id,
            education_level_id: level.id,
            subject_id: subject.id,
            school_name: None,
            district_id: None,
            province_id: None,
        },
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("already registered") || err.contains("Conflict"),
        "Expected conflict error, got: {}",
        err
    );

    cleanup_full(&db, event.id, level.id, subject.id, user_id).await;
}

/// Cannot transition out of order (e.g., registered → in_progress)
#[tokio::test]
async fn peserta_invalid_status_transition() {
    let db = TestDb::new().await;

    let level = EventRepository::create_education_level(&db.pool, "SMA Trans")
        .await
        .unwrap();
    let subject = EventRepository::create_subject(&db.pool, "Math Trans")
        .await
        .unwrap();
    let event = EventRepository::create_event(&db.pool, "Trans Test", "2025/2026")
        .await
        .unwrap();
    let stage = EventRepository::create_stage(&db.pool, event.id, Tier::District)
        .await
        .unwrap();
    let user_id = db.create_test_user("trans-peserta").await;

    let participant = ParticipantRepository::register(
        &db.pool,
        event.id,
        &RegisterParticipantRequest {
            user_id,
            education_level_id: level.id,
            subject_id: subject.id,
            school_name: None,
            district_id: None,
            province_id: None,
        },
    )
    .await
    .unwrap();

    let ps = ParticipantRepository::create_stage_entry(&db.pool, participant.id, stage.id)
        .await
        .unwrap();
    assert_eq!(ps.status, "registered");

    // Try invalid: registered → in_progress (should be registered → verified)
    let result = ParticipantRepository::transition_stage_status(&db.pool, ps.id, "in_progress").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Cannot transition"), "Expected transition error, got: {}", err);

    // Try invalid: registered → scored
    let result = ParticipantRepository::transition_stage_status(&db.pool, ps.id, "scored").await;
    assert!(result.is_err());

    // Valid: registered → verified → assigned_to_exam
    ParticipantRepository::transition_stage_status(&db.pool, ps.id, "verified")
        .await
        .unwrap();
    ParticipantRepository::transition_stage_status(&db.pool, ps.id, "assigned_to_exam")
        .await
        .unwrap();

    cleanup_full(&db, event.id, level.id, subject.id, user_id).await;
}

/// Answer upsert: re-answering same question updates instead of duplicating
#[tokio::test]
async fn peserta_can_change_answer() {
    let db = TestDb::new().await;

    let level = EventRepository::create_education_level(&db.pool, "SMA Upsert")
        .await
        .unwrap();
    let subject = EventRepository::create_subject(&db.pool, "Math Upsert")
        .await
        .unwrap();
    let event = EventRepository::create_event(&db.pool, "Upsert Test", "2025/2026")
        .await
        .unwrap();
    let stage = EventRepository::create_stage(&db.pool, event.id, Tier::District)
        .await
        .unwrap();
    let user_id = db.create_test_user("upsert-peserta").await;

    let participant = ParticipantRepository::register(
        &db.pool,
        event.id,
        &RegisterParticipantRequest {
            user_id,
            education_level_id: level.id,
            subject_id: subject.id,
            school_name: None,
            district_id: None,
            province_id: None,
        },
    )
    .await
    .unwrap();

    let ps = ParticipantRepository::create_stage_entry(&db.pool, participant.id, stage.id)
        .await
        .unwrap();

    // Walk to in_progress
    ParticipantRepository::transition_stage_status(&db.pool, ps.id, "verified").await.unwrap();
    ParticipantRepository::transition_stage_status(&db.pool, ps.id, "assigned_to_exam").await.unwrap();

    let exam = ExamRepository::create(
        &db.pool,
        stage.id,
        &CreateExamRequest {
            title: "Upsert Exam".into(),
            description: None,
            duration_minutes: 60,
            max_attempts: None,
            shuffle_questions: None,
            shuffle_options: None,
            opens_at: None,
            closes_at: None,
        },
    )
    .await
    .unwrap();

    let q1 = ExamRepository::create_question(
        &db.pool,
        exam.id,
        &CreateQuestionRequest {
            question_text: "1+1?".into(),
            question_type: "multiple_choice".into(),
            options: Some(serde_json::json!(["1", "2", "3"])),
            correct_answer: Some(serde_json::json!("2")),
            points: 10.0,
            sequence: 1,
        },
    )
    .await
    .unwrap();

    let session = ExamRepository::assign_session(&db.pool, ps.id, exam.id).await.unwrap();
    ParticipantRepository::transition_stage_status(&db.pool, ps.id, "in_progress").await.unwrap();
    ExamRepository::start_session(&db.pool, session.id).await.unwrap();

    // First answer: wrong
    ExamRepository::save_answer(&db.pool, session.id, q1.id, &serde_json::json!("1"))
        .await
        .unwrap();

    // Change answer: correct
    ExamRepository::save_answer(&db.pool, session.id, q1.id, &serde_json::json!("2"))
        .await
        .unwrap();

    // Should have only 1 answer (upsert, not duplicate)
    let answers = ExamRepository::list_answers(&db.pool, session.id).await.unwrap();
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0].answer_data, Some(serde_json::json!("2")));

    // Submit — should score correctly (changed to correct answer)
    let result = ExamRepository::submit_session(&db.pool, session.id, false).await.unwrap();
    assert_eq!(result.total_score, 10.0);
    assert_eq!(result.correct_count, 1);

    cleanup_full(&db, event.id, level.id, subject.id, user_id).await;
}

/// Participants listed by stage with rank ordering
#[tokio::test]
async fn list_participants_by_stage() {
    let db = TestDb::new().await;

    let level = EventRepository::create_education_level(&db.pool, "SMA List")
        .await
        .unwrap();
    let subject = EventRepository::create_subject(&db.pool, "Math List")
        .await
        .unwrap();
    let event = EventRepository::create_event(&db.pool, "List Test", "2025/2026")
        .await
        .unwrap();
    let stage = EventRepository::create_stage(&db.pool, event.id, Tier::District)
        .await
        .unwrap();

    // Register 3 participants
    let mut user_ids = vec![];
    for i in 0..3 {
        let uid = db.create_test_user(&format!("list-peserta-{}", i)).await;
        user_ids.push(uid);

        let p = ParticipantRepository::register(
            &db.pool,
            event.id,
            &RegisterParticipantRequest {
                user_id: uid,
                education_level_id: level.id,
                subject_id: subject.id,
                school_name: Some(format!("School {}", i)),
                district_id: None,
                province_id: None,
            },
        )
        .await
        .unwrap();

        ParticipantRepository::create_stage_entry(&db.pool, p.id, stage.id)
            .await
            .unwrap();
    }

    // Count
    let count = ParticipantRepository::count_by_stage(&db.pool, stage.id).await.unwrap();
    assert_eq!(count, 3);

    // List
    let list = ParticipantRepository::list_by_stage(&db.pool, stage.id, 10, 0).await.unwrap();
    assert_eq!(list.len(), 3);
    assert!(list.iter().all(|p| p.stage_status == Some("registered".into())));

    // Count by event
    let event_count = ParticipantRepository::count_by_event(&db.pool, event.id).await.unwrap();
    assert_eq!(event_count, 3);

    cleanup_full(&db, event.id, level.id, subject.id, uuid::Uuid::nil()).await;
    for uid in user_ids {
        db.cleanup_user(uid).await;
    }
}

// ─── Cleanup helper ───

async fn cleanup_full(db: &TestDb, event_id: uuid::Uuid, level_id: uuid::Uuid, subject_id: uuid::Uuid, user_id: uuid::Uuid) {
    // Delete in dependency order
    sqlx::query("DELETE FROM answers WHERE exam_session_id IN (SELECT id FROM exam_sessions WHERE exam_id IN (SELECT id FROM exams WHERE stage_id IN (SELECT id FROM stages WHERE event_id = $1)))")
        .bind(event_id)
        .execute(&db.pool)
        .await
        .ok();
    sqlx::query("DELETE FROM exam_sessions WHERE exam_id IN (SELECT id FROM exams WHERE stage_id IN (SELECT id FROM stages WHERE event_id = $1))")
        .bind(event_id)
        .execute(&db.pool)
        .await
        .ok();
    sqlx::query("DELETE FROM questions WHERE exam_id IN (SELECT id FROM exams WHERE stage_id IN (SELECT id FROM stages WHERE event_id = $1))")
        .bind(event_id)
        .execute(&db.pool)
        .await
        .ok();
    sqlx::query("DELETE FROM exams WHERE stage_id IN (SELECT id FROM stages WHERE event_id = $1)")
        .bind(event_id)
        .execute(&db.pool)
        .await
        .ok();
    sqlx::query("DELETE FROM participant_stages WHERE participant_id IN (SELECT id FROM participants WHERE event_id = $1)")
        .bind(event_id)
        .execute(&db.pool)
        .await
        .ok();
    sqlx::query("DELETE FROM participants WHERE event_id = $1")
        .bind(event_id)
        .execute(&db.pool)
        .await
        .ok();
    db.cleanup_event(event_id).await;
    db.cleanup_education_level(level_id).await;
    db.cleanup_subject(subject_id).await;
    if user_id != uuid::Uuid::nil() {
        db.cleanup_user(user_id).await;
    }
}
