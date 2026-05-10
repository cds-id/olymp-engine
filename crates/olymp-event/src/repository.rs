use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use olymp_core::types::StageStatus;
use olymp_core::AppError;

pub struct EventRepository;

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

impl EventRepository {
    // ─── Education Levels ───

    pub async fn list_education_levels(pool: &PgPool) -> Result<Vec<EducationLevel>, AppError> {
        sqlx::query_as::<_, EducationLevel>("SELECT * FROM education_levels ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn create_education_level(
        pool: &PgPool,
        name: &str,
    ) -> Result<EducationLevel, AppError> {
        let slug = slugify(name);
        sqlx::query_as::<_, EducationLevel>(
            "INSERT INTO education_levels (name, slug) VALUES ($1, $2) RETURNING *",
        )
        .bind(name)
        .bind(&slug)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Subjects ───

    pub async fn list_subjects(pool: &PgPool) -> Result<Vec<Subject>, AppError> {
        sqlx::query_as::<_, Subject>("SELECT * FROM subjects ORDER BY name")
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn create_subject(pool: &PgPool, name: &str) -> Result<Subject, AppError> {
        let slug = slugify(name);
        sqlx::query_as::<_, Subject>(
            "INSERT INTO subjects (name, slug) VALUES ($1, $2) RETURNING *",
        )
        .bind(name)
        .bind(&slug)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Events ───

    pub async fn list_events(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Event>, AppError> {
        sqlx::query_as::<_, Event>(
            "SELECT * FROM events ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn count_events(pool: &PgPool) -> Result<i64, AppError> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM events")
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn get_event(pool: &PgPool, id: Uuid) -> Result<Option<Event>, AppError> {
        sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn create_event(
        pool: &PgPool,
        name: &str,
        academic_year: &str,
    ) -> Result<Event, AppError> {
        let slug = slugify(name);
        sqlx::query_as::<_, Event>(
            "INSERT INTO events (name, slug, academic_year) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(name)
        .bind(&slug)
        .bind(academic_year)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update_event(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateEventRequest,
    ) -> Result<Event, AppError> {
        let current = Self::get_event(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Event not found".into()))?;

        let name = req.name.as_deref().unwrap_or(&current.name);
        let academic_year = req
            .academic_year
            .as_deref()
            .unwrap_or(&current.academic_year);
        let status = req
            .status
            .map(|s| s.to_string())
            .unwrap_or(current.status);

        sqlx::query_as::<_, Event>(
            "UPDATE events SET name = $2, academic_year = $3, status = $4, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(name)
        .bind(academic_year)
        .bind(&status)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Stages ───

    pub async fn list_stages(pool: &PgPool, event_id: Uuid) -> Result<Vec<Stage>, AppError> {
        sqlx::query_as::<_, Stage>(
            "SELECT * FROM stages WHERE event_id = $1 ORDER BY sequence",
        )
        .bind(event_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn get_stage(pool: &PgPool, id: Uuid) -> Result<Option<Stage>, AppError> {
        sqlx::query_as::<_, Stage>("SELECT * FROM stages WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn create_stage(
        pool: &PgPool,
        event_id: Uuid,
        req: &CreateStageRequest,
    ) -> Result<Stage, AppError> {
        let sequence = req.tier.sequence();
        sqlx::query_as::<_, Stage>(
            "INSERT INTO stages (event_id, tier, sequence, name, location, district_id, province_id, capacity,
                                registration_opens_at, registration_closes_at, started_at, ended_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING *",
        )
        .bind(event_id)
        .bind(req.tier.to_string())
        .bind(sequence)
        .bind(&req.name)
        .bind(&req.location)
        .bind(req.district_id)
        .bind(req.province_id)
        .bind(req.capacity)
        .bind(req.registration_opens_at)
        .bind(req.registration_closes_at)
        .bind(req.started_at)
        .bind(req.ended_at)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update_stage(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateStageRequest,
    ) -> Result<Stage, AppError> {
        let current = Self::get_stage(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Stage not found".into()))?;

        sqlx::query_as::<_, Stage>(
            "UPDATE stages SET
               name = $2, location = $3, district_id = $4, province_id = $5, capacity = $6,
               registration_opens_at = $7, registration_closes_at = $8,
               started_at = $9, ended_at = $10, updated_at = now()
             WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(req.name.as_deref().or(current.name.as_deref()))
        .bind(req.location.as_deref().or(current.location.as_deref()))
        .bind(req.district_id.or(current.district_id))
        .bind(req.province_id.or(current.province_id))
        .bind(req.capacity.or(current.capacity))
        .bind(req.registration_opens_at.or(current.registration_opens_at))
        .bind(req.registration_closes_at.or(current.registration_closes_at))
        .bind(req.started_at.or(current.started_at))
        .bind(req.ended_at.or(current.ended_at))
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update_stage_status(
        pool: &PgPool,
        id: Uuid,
        status: StageStatus,
    ) -> Result<Stage, AppError> {
        sqlx::query_as::<_, Stage>(
            "UPDATE stages SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(status.to_string())
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    /// List stages open for registration with enrollment counts (peserta-facing)
    pub async fn list_available_stages(
        pool: &PgPool,
        event_id: Uuid,
        filters: &AvailableStageFilters,
    ) -> Result<Vec<StageWithEnrollment>, AppError> {
        sqlx::query_as::<_, StageWithEnrollment>(
            "SELECT s.*,
                    COALESCE((SELECT COUNT(*) FROM participant_stages ps WHERE ps.stage_id = s.id), 0) AS enrolled_count
             FROM stages s
             WHERE s.event_id = $1
               AND (s.registration_opens_at IS NULL OR s.registration_opens_at <= now())
               AND (s.registration_closes_at IS NULL OR s.registration_closes_at >= now())
               AND ($2::TEXT IS NULL OR s.tier = $2)
               AND ($3::UUID IS NULL OR s.province_id = $3)
               AND ($4::UUID IS NULL OR s.district_id = $4)
             ORDER BY s.sequence, s.name",
        )
        .bind(event_id)
        .bind(&filters.tier)
        .bind(filters.province_id)
        .bind(filters.district_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Event Categories ───

    pub async fn list_event_categories(
        pool: &PgPool,
        event_id: Uuid,
    ) -> Result<Vec<EventCategory>, AppError> {
        sqlx::query_as::<_, EventCategory>(
            "SELECT * FROM event_categories WHERE event_id = $1",
        )
        .bind(event_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn create_event_category(
        pool: &PgPool,
        event_id: Uuid,
        education_level_id: Uuid,
        subject_id: Uuid,
    ) -> Result<EventCategory, AppError> {
        sqlx::query_as::<_, EventCategory>(
            "INSERT INTO event_categories (event_id, education_level_id, subject_id) VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(event_id)
        .bind(education_level_id)
        .bind(subject_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }
}
