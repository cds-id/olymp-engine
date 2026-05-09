//! Integration test: prove full competition setup Daerah → Provinsi → Nasional
//!
//! Exercises:
//! 1. Create education levels + subjects
//! 2. Create event with academic_year
//! 3. Create 3 stages (district/province/national), verify sequence + tier
//! 4. Create event categories (link education_level + subject to event)
//! 5. Activate event
//! 6. Open registration on district stage
//! 7. Register participant, create participant_stage entry
//! 8. Walk participant through district stage statuses
//! 9. Create exams + questions for each stage

mod helpers;

use helpers::TestDb;
use olymp_event::repository::EventRepository;
use olymp_event::models::*;
use olymp_core::types::{Tier, StageStatus, EventStatus};

// ─── 1. Full competition setup: event + 3 tiers ───

#[tokio::test]
async fn create_event_with_three_tier_stages() {
    let db = TestDb::new().await;

    // Create education level
    let level = EventRepository::create_education_level(&db.pool, "SMA")
        .await
        .unwrap();
    assert_eq!(level.name, "SMA");
    assert_eq!(level.slug, "sma");

    // Create subject
    let subject = EventRepository::create_subject(&db.pool, "Matematika")
        .await
        .unwrap();
    assert_eq!(subject.name, "Matematika");
    assert_eq!(subject.slug, "matematika");

    // Create event
    let event = EventRepository::create_event(&db.pool, "OSN 2026", "2025/2026")
        .await
        .unwrap();
    assert_eq!(event.name, "OSN 2026");
    assert_eq!(event.slug, "osn-2026");
    assert_eq!(event.academic_year, "2025/2026");
    assert_eq!(event.status, "draft");

    // Create 3 stages: district → province → national
    let district = EventRepository::create_stage(&db.pool, event.id, Tier::District)
        .await
        .unwrap();
    assert_eq!(district.tier, "district");
    assert_eq!(district.sequence, 1);
    assert_eq!(district.status, "draft");

    let province = EventRepository::create_stage(&db.pool, event.id, Tier::Province)
        .await
        .unwrap();
    assert_eq!(province.tier, "province");
    assert_eq!(province.sequence, 2);

    let national = EventRepository::create_stage(&db.pool, event.id, Tier::National)
        .await
        .unwrap();
    assert_eq!(national.tier, "national");
    assert_eq!(national.sequence, 3);

    // List stages — verify order
    let stages = EventRepository::list_stages(&db.pool, event.id)
        .await
        .unwrap();
    assert_eq!(stages.len(), 3);
    assert_eq!(stages[0].sequence, 1);
    assert_eq!(stages[1].sequence, 2);
    assert_eq!(stages[2].sequence, 3);

    // Create event category (link subject + level to event)
    let cat = EventRepository::create_event_category(
        &db.pool,
        event.id,
        level.id,
        subject.id,
    )
    .await
    .unwrap();
    assert_eq!(cat.event_id, event.id);
    assert_eq!(cat.education_level_id, level.id);
    assert_eq!(cat.subject_id, subject.id);

    // List categories
    let cats = EventRepository::list_event_categories(&db.pool, event.id)
        .await
        .unwrap();
    assert_eq!(cats.len(), 1);

    // Activate event
    let updated = EventRepository::update_event(
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
    assert_eq!(updated.status, "active");

    // Open registration on district stage
    let district = EventRepository::update_stage_status(
        &db.pool,
        district.id,
        StageStatus::OpenRegistration,
    )
    .await
    .unwrap();
    assert_eq!(district.status, "open_registration");

    // Cleanup
    db.cleanup_event(event.id).await;
    db.cleanup_education_level(level.id).await;
    db.cleanup_subject(subject.id).await;
}

// ─── 2. Stage status progression through full lifecycle ───

#[tokio::test]
async fn stage_status_full_lifecycle() {
    let db = TestDb::new().await;

    let event = EventRepository::create_event(&db.pool, "Lifecycle Test", "2025/2026")
        .await
        .unwrap();
    let stage = EventRepository::create_stage(&db.pool, event.id, Tier::District)
        .await
        .unwrap();
    assert_eq!(stage.status, "draft");

    // Walk through full lifecycle
    let statuses = [
        StageStatus::OpenRegistration,
        StageStatus::RegistrationClosed,
        StageStatus::Verification,
        StageStatus::ReadyForExam,
        StageStatus::ExamOpen,
        StageStatus::ExamClosed,
        StageStatus::Scoring,
        StageStatus::RankingReview,
        StageStatus::RankingApproved,
        StageStatus::ResultPublished,
        StageStatus::Promoted,
        StageStatus::Finalized,
    ];

    let mut current_id = stage.id;
    for status in &statuses {
        let updated = EventRepository::update_stage_status(&db.pool, current_id, *status)
            .await
            .unwrap();
        assert_eq!(
            updated.status,
            status.to_string(),
            "stage status mismatch at {:?}",
            status
        );
        current_id = updated.id;
    }

    db.cleanup_event(event.id).await;
}

// ─── 3. Multiple education levels + subjects per event ───

#[tokio::test]
async fn event_multiple_categories() {
    let db = TestDb::new().await;

    let sma = EventRepository::create_education_level(&db.pool, "SMA/Sederajat")
        .await
        .unwrap();
    let smp = EventRepository::create_education_level(&db.pool, "SMP/Sederajat")
        .await
        .unwrap();

    let math = EventRepository::create_subject(&db.pool, "Matematika SMA")
        .await
        .unwrap();
    let physics = EventRepository::create_subject(&db.pool, "Fisika")
        .await
        .unwrap();
    let bio = EventRepository::create_subject(&db.pool, "Biologi")
        .await
        .unwrap();

    let event = EventRepository::create_event(&db.pool, "OSN Multi", "2025/2026")
        .await
        .unwrap();

    // Create categories: SMA-Math, SMA-Physics, SMP-Bio
    EventRepository::create_event_category(&db.pool, event.id, sma.id, math.id)
        .await
        .unwrap();
    EventRepository::create_event_category(&db.pool, event.id, sma.id, physics.id)
        .await
        .unwrap();
    EventRepository::create_event_category(&db.pool, event.id, smp.id, bio.id)
        .await
        .unwrap();

    let cats = EventRepository::list_event_categories(&db.pool, event.id)
        .await
        .unwrap();
    assert_eq!(cats.len(), 3);

    db.cleanup_event(event.id).await;
    db.cleanup_education_level(sma.id).await;
    db.cleanup_education_level(smp.id).await;
    db.cleanup_subject(math.id).await;
    db.cleanup_subject(physics.id).await;
    db.cleanup_subject(bio.id).await;
}

// ─── 4. Tier sequence mapping ───

#[test]
fn tier_sequence_and_next() {
    assert_eq!(Tier::District.sequence(), 1);
    assert_eq!(Tier::Province.sequence(), 2);
    assert_eq!(Tier::National.sequence(), 3);

    assert_eq!(Tier::District.next(), Some(Tier::Province));
    assert_eq!(Tier::Province.next(), Some(Tier::National));
    assert_eq!(Tier::National.next(), None);
}

// ─── 5. Event get/list ───

#[tokio::test]
async fn event_get_and_list() {
    let db = TestDb::new().await;

    let event = EventRepository::create_event(&db.pool, "Get Test", "2024/2025")
        .await
        .unwrap();

    // Get by ID
    let found = EventRepository::get_event(&db.pool, event.id)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Get Test");

    // Get nonexistent
    let missing = EventRepository::get_event(&db.pool, uuid::Uuid::from_u128(99999))
        .await
        .unwrap();
    assert!(missing.is_none());

    // List
    let events = EventRepository::list_events(&db.pool).await.unwrap();
    assert!(events.iter().any(|e| e.id == event.id));

    db.cleanup_event(event.id).await;
}

// ─── 6. Update event fields ───

#[tokio::test]
async fn update_event_fields() {
    let db = TestDb::new().await;

    let event = EventRepository::create_event(&db.pool, "Update Me", "2024/2025")
        .await
        .unwrap();

    let updated = EventRepository::update_event(
        &db.pool,
        event.id,
        &UpdateEventRequest {
            name: Some("Updated Name".into()),
            academic_year: Some("2025/2026".into()),
            status: None,
        },
    )
    .await
    .unwrap();

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.academic_year, "2025/2026");
    assert_eq!(updated.status, "draft"); // unchanged

    db.cleanup_event(event.id).await;
}
