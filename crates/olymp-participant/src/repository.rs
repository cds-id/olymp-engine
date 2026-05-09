use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::{Participant, TierProgression};

pub struct ParticipantRepository;

impl ParticipantRepository {
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        tier: &str,
    ) -> Result<Participant, sqlx::Error> {
        sqlx::query_as::<_, Participant>(
            r#"
            INSERT INTO participants (id, user_id, current_tier, is_locked, score, created_at, updated_at)
            VALUES ($1, $2, $3, false, 0.0, $4, $5)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(tier)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(pool)
        .await
    }

    pub async fn get_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Option<Participant>, sqlx::Error> {
        sqlx::query_as::<_, Participant>(
            "SELECT * FROM participants WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    pub async fn get_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<Participant>, sqlx::Error> {
        sqlx::query_as::<_, Participant>(
            "SELECT * FROM participants WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
    }

    pub async fn lock_participant(
        pool: &PgPool,
        participant_id: Uuid,
        account_name: &str,
    ) -> Result<Participant, sqlx::Error> {
        sqlx::query_as::<_, Participant>(
            r#"
            UPDATE participants
            SET is_locked = true, locked_by_account = $2, updated_at = $3
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(participant_id)
        .bind(account_name)
        .bind(Utc::now())
        .fetch_one(pool)
        .await
    }

    pub async fn unlock_participant(
        pool: &PgPool,
        participant_id: Uuid,
    ) -> Result<Participant, sqlx::Error> {
        sqlx::query_as::<_, Participant>(
            r#"
            UPDATE participants
            SET is_locked = false, locked_by_account = NULL, updated_at = $2
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(participant_id)
        .bind(Utc::now())
        .fetch_one(pool)
        .await
    }

    pub async fn update_score(
        pool: &PgPool,
        participant_id: Uuid,
        score: f64,
    ) -> Result<Participant, sqlx::Error> {
        sqlx::query_as::<_, Participant>(
            r#"
            UPDATE participants
            SET score = $2, updated_at = $3
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(participant_id)
        .bind(score)
        .bind(Utc::now())
        .fetch_one(pool)
        .await
    }

    pub async fn record_tier_progression(
        pool: &PgPool,
        participant_id: Uuid,
        from_tier: &str,
        to_tier: &str,
    ) -> Result<TierProgression, sqlx::Error> {
        sqlx::query_as::<_, TierProgression>(
            r#"
            INSERT INTO tier_progressions (id, participant_id, from_tier, to_tier, advanced_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(participant_id)
        .bind(from_tier)
        .bind(to_tier)
        .bind(Utc::now())
        .fetch_one(pool)
        .await
    }

    pub async fn get_tier_history(
        pool: &PgPool,
        participant_id: Uuid,
    ) -> Result<Vec<TierProgression>, sqlx::Error> {
        sqlx::query_as::<_, TierProgression>(
            "SELECT * FROM tier_progressions WHERE participant_id = $1 ORDER BY advanced_at DESC"
        )
        .bind(participant_id)
        .fetch_all(pool)
        .await
    }
}
