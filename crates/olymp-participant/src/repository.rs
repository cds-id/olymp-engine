use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use olymp_core::AppError;

pub struct ParticipantRepository;

impl ParticipantRepository {
    // ─── Participants ───

    pub async fn register(
        pool: &PgPool,
        event_id: Uuid,
        req: &RegisterParticipantRequest,
    ) -> Result<Participant, AppError> {
        sqlx::query_as::<_, Participant>(
            "INSERT INTO participants (user_id, event_id, education_level_id, subject_id, school_name, district_id, province_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING *",
        )
        .bind(req.user_id)
        .bind(event_id)
        .bind(req.education_level_id)
        .bind(req.subject_id)
        .bind(&req.school_name)
        .bind(req.district_id)
        .bind(req.province_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db) if db.constraint().is_some() => {
                AppError::Conflict("Participant already registered for this event/subject".into())
            }
            other => AppError::Database(other),
        })
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Participant>, AppError> {
        sqlx::query_as::<_, Participant>("SELECT * FROM participants WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        req: &UpdateParticipantRequest,
    ) -> Result<Participant, AppError> {
        let current = Self::get_by_id(pool, id)
            .await?
            .ok_or_else(|| AppError::NotFound("Participant not found".into()))?;

        let school_name = req.school_name.as_deref().or(current.school_name.as_deref());
        let district_id = req.district_id.or(current.district_id);
        let province_id = req.province_id.or(current.province_id);

        sqlx::query_as::<_, Participant>(
            "UPDATE participants SET school_name = $2, district_id = $3, province_id = $4, updated_at = now()
             WHERE id = $1 RETURNING *",
        )
        .bind(id)
        .bind(school_name)
        .bind(district_id)
        .bind(province_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn list_by_event(
        pool: &PgPool,
        event_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Participant>, AppError> {
        sqlx::query_as::<_, Participant>(
            "SELECT * FROM participants WHERE event_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(event_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn count_by_event(pool: &PgPool, event_id: Uuid) -> Result<i64, AppError> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM participants WHERE event_id = $1",
        )
        .bind(event_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Participant Stages ───

    pub async fn create_stage_entry(
        pool: &PgPool,
        participant_id: Uuid,
        stage_id: Uuid,
    ) -> Result<ParticipantStage, AppError> {
        sqlx::query_as::<_, ParticipantStage>(
            "INSERT INTO participant_stages (participant_id, stage_id)
             VALUES ($1, $2) RETURNING *",
        )
        .bind(participant_id)
        .bind(stage_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(ref db) if db.constraint().is_some() => {
                AppError::Conflict("Participant already has entry for this stage".into())
            }
            other => AppError::Database(other),
        })
    }

    pub async fn get_stage_entry(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<ParticipantStage>, AppError> {
        sqlx::query_as::<_, ParticipantStage>(
            "SELECT * FROM participant_stages WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn get_participant_stages(
        pool: &PgPool,
        participant_id: Uuid,
    ) -> Result<Vec<ParticipantStage>, AppError> {
        sqlx::query_as::<_, ParticipantStage>(
            "SELECT * FROM participant_stages WHERE participant_id = $1 ORDER BY created_at",
        )
        .bind(participant_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn transition_stage_status(
        pool: &PgPool,
        ps_id: Uuid,
        new_status: &str,
    ) -> Result<ParticipantStage, AppError> {
        let current = Self::get_stage_entry(pool, ps_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Participant stage entry not found".into()))?;

        if !current.can_transition_to(new_status) {
            return Err(AppError::BadRequest(format!(
                "Cannot transition from '{}' to '{}'",
                current.status, new_status
            )));
        }

        let promoted_at = if new_status == "qualified" {
            Some(chrono::Utc::now())
        } else {
            current.promoted_at
        };

        sqlx::query_as::<_, ParticipantStage>(
            "UPDATE participant_stages SET status = $2, promoted_at = $3, updated_at = now()
             WHERE id = $1 RETURNING *",
        )
        .bind(ps_id)
        .bind(new_status)
        .bind(promoted_at)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    /// List participants for a stage with their stage-specific data
    pub async fn list_by_stage(
        pool: &PgPool,
        stage_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ParticipantListItem>, AppError> {
        sqlx::query_as::<_, ParticipantListItem>(
            "SELECT p.id, p.user_id, p.school_name, p.district_id, p.province_id,
                    ps.status AS stage_status, ps.score, ps.rank
             FROM participants p
             JOIN participant_stages ps ON ps.participant_id = p.id
             WHERE ps.stage_id = $1
             ORDER BY ps.rank NULLS LAST, ps.score DESC NULLS LAST
             LIMIT $2 OFFSET $3",
        )
        .bind(stage_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn count_by_stage(pool: &PgPool, stage_id: Uuid) -> Result<i64, AppError> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM participant_stages WHERE stage_id = $1",
        )
        .bind(stage_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Batch Operations ───

    /// Batch transition participant_stages in a stage from one status to another.
    /// If participant_ids is None/empty, applies to ALL with matching from_status.
    pub async fn batch_transition(
        pool: &PgPool,
        stage_id: Uuid,
        participant_ids: Option<&[Uuid]>,
        from_status: &str,
        to_status: &str,
    ) -> Result<crate::models::BatchResult, AppError> {
        // Get eligible participant_stages
        let stages = if let Some(ids) = participant_ids.filter(|ids| !ids.is_empty()) {
            sqlx::query_as::<_, ParticipantStage>(
                "SELECT ps.* FROM participant_stages ps
                 JOIN participants p ON p.id = ps.participant_id
                 WHERE ps.stage_id = $1 AND ps.status = $2
                   AND p.id = ANY($3)
                 ORDER BY ps.created_at",
            )
            .bind(stage_id)
            .bind(from_status)
            .bind(ids)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?
        } else {
            sqlx::query_as::<_, ParticipantStage>(
                "SELECT ps.* FROM participant_stages ps
                 WHERE ps.stage_id = $1 AND ps.status = $2
                 ORDER BY ps.created_at",
            )
            .bind(stage_id)
            .bind(from_status)
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?
        };

        let mut affected = 0i32;
        let mut skipped = 0i32;
        let mut errors = Vec::new();

        for ps in &stages {
            if !ps.can_transition_to(to_status) {
                skipped += 1;
                errors.push(format!(
                    "participant_stage {} cannot transition from '{}' to '{}'",
                    ps.id, ps.status, to_status
                ));
                continue;
            }

            let promoted_at = if to_status == "qualified" {
                Some(chrono::Utc::now())
            } else {
                ps.promoted_at
            };

            match sqlx::query(
                "UPDATE participant_stages SET status = $2, promoted_at = $3, updated_at = now()
                 WHERE id = $1",
            )
            .bind(ps.id)
            .bind(to_status)
            .bind(promoted_at)
            .execute(pool)
            .await
            {
                Ok(_) => affected += 1,
                Err(e) => {
                    skipped += 1;
                    errors.push(format!("participant_stage {}: {}", ps.id, e));
                }
            }
        }

        Ok(crate::models::BatchResult {
            affected,
            skipped,
            errors,
        })
    }

    /// List all participations for a user (across all events)
    pub async fn list_by_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<Participant>, AppError> {
        sqlx::query_as::<_, Participant>(
            "SELECT * FROM participants WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }
}
