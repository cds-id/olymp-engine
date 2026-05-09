use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Participant, RegisterParticipantRequest, AdvanceTierRequest};
use crate::repository::ParticipantRepository;

pub struct ParticipantService;

impl ParticipantService {
    pub async fn register_participant(
        pool: &PgPool,
        user_id: Uuid,
        req: RegisterParticipantRequest,
    ) -> Result<Participant, String> {
        match req.tier.as_str() {
            "district" | "province" | "national" => {},
            _ => return Err("Invalid tier".to_string()),
        }

        if let Ok(Some(_)) = ParticipantRepository::get_by_user_id(pool, user_id).await {
            return Err("User already registered as participant".to_string());
        }

        ParticipantRepository::create(pool, user_id, &req.tier)
            .await
            .map_err(|e| format!("Database error: {}", e))
    }

    pub async fn get_participant_profile(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Participant, String> {
        ParticipantRepository::get_by_user_id(pool, user_id)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Participant not found".to_string())
    }

    pub async fn advance_tier(
        pool: &PgPool,
        participant_id: Uuid,
        req: AdvanceTierRequest,
    ) -> Result<Participant, String> {
        let participant = ParticipantRepository::get_by_id(pool, participant_id)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Participant not found".to_string())?;

        let valid_progression = match participant.current_tier.as_str() {
            "district" => req.to_tier == "province",
            "province" => req.to_tier == "national",
            _ => false,
        };

        if !valid_progression {
            return Err("Invalid tier progression".to_string());
        }

        ParticipantRepository::record_tier_progression(
            pool,
            participant_id,
            &participant.current_tier,
            &req.to_tier,
        )
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        sqlx::query(
            "UPDATE participants SET current_tier = $1, is_locked = false, locked_by_account = NULL WHERE id = $2"
        )
        .bind(&req.to_tier)
        .bind(participant_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

        ParticipantRepository::get_by_id(pool, participant_id)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Participant not found".to_string())
    }

    pub async fn lock_for_approval(
        pool: &PgPool,
        participant_id: Uuid,
        account_name: &str,
    ) -> Result<Participant, String> {
        ParticipantRepository::lock_participant(pool, participant_id, account_name)
            .await
            .map_err(|e| format!("Database error: {}", e))
    }
}
