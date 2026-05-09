use sqlx::PgPool;
use uuid::Uuid;

use crate::models::*;
use olymp_core::AppError;

pub struct RankingRepository;

impl RankingRepository {
    // ─── Ranking Rules ───

    pub async fn get_rule_by_stage(
        pool: &PgPool,
        stage_id: Uuid,
    ) -> Result<Option<RankingRule>, AppError> {
        sqlx::query_as::<_, RankingRule>("SELECT * FROM ranking_rules WHERE stage_id = $1")
            .bind(stage_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn upsert_rule(
        pool: &PgPool,
        stage_id: Uuid,
        req: &CreateRankingRuleRequest,
    ) -> Result<RankingRule, AppError> {
        let tiebreaker = req
            .tiebreaker_order
            .as_ref()
            .map(|v| serde_json::json!(v))
            .unwrap_or_else(|| serde_json::json!(["score_desc", "time_asc", "cheating_asc"]));

        sqlx::query_as::<_, RankingRule>(
            "INSERT INTO ranking_rules (stage_id, max_qualifiers, min_score, max_cheating_logs, tiebreaker_order)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (stage_id)
             DO UPDATE SET max_qualifiers = $2, min_score = $3, max_cheating_logs = $4,
                           tiebreaker_order = $5, updated_at = now()
             RETURNING *",
        )
        .bind(stage_id)
        .bind(req.max_qualifiers)
        .bind(req.min_score)
        .bind(req.max_cheating_logs)
        .bind(&tiebreaker)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Calculate Ranking ───

    /// Calculate ranking for a stage: reads participant_stages, sorts by tiebreaker, creates ranking_result + entries
    pub async fn calculate(
        pool: &PgPool,
        stage_id: Uuid,
    ) -> Result<RankingResultWithEntries, AppError> {
        let rule = Self::get_rule_by_stage(pool, stage_id)
            .await?
            .ok_or_else(|| AppError::NotFound("No ranking rule for this stage".into()))?;

        // Parse tiebreaker order
        let tiebreakers: Vec<String> = serde_json::from_value(rule.tiebreaker_order.clone())
            .unwrap_or_else(|_| vec!["score_desc".into(), "time_asc".into()]);

        // Load all submitted participant_stages for this stage
        let candidates = sqlx::query_as::<_, RankCandidate>(
            "SELECT ps.id as participant_stage_id, COALESCE(ps.score, 0) as score,
                    ps.completion_time_secs, COALESCE(ps.cheating_log_count, 0) as cheating_log_count
             FROM participant_stages ps
             WHERE ps.stage_id = $1
               AND ps.status IN ('submitted', 'scored', 'ranked', 'qualified', 'not_qualified')",
        )
        .bind(stage_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        if candidates.is_empty() {
            return Err(AppError::BadRequest("No participants to rank".into()));
        }

        // Sort candidates
        let mut sorted = candidates;
        sorted.sort_by(|a, b| RankCandidate::compare_by_tiebreakers(a, b, &tiebreakers));

        // Apply qualification rules
        let total_participants = sorted.len() as i32;
        let mut total_qualified = 0i32;

        let entries_with_status: Vec<(usize, &RankCandidate, String)> = sorted
            .iter()
            .enumerate()
            .map(|(idx, c)| {
                let rank = (idx + 1) as i32;

                // Disqualify if too many cheating logs
                if let Some(max_cheat) = rule.max_cheating_logs {
                    if c.cheating_log_count > max_cheat {
                        return (idx, c, "disqualified".to_string());
                    }
                }

                // Check min_score
                if let Some(min) = rule.min_score {
                    if c.score < min {
                        return (idx, c, "not_qualified".to_string());
                    }
                }

                // Check max_qualifiers
                let status = if let Some(max_q) = rule.max_qualifiers {
                    if rank <= max_q {
                        total_qualified += 1;
                        "qualified"
                    } else {
                        "not_qualified"
                    }
                } else {
                    // No max — all passing min_score are qualified
                    total_qualified += 1;
                    "qualified"
                };

                (idx, c, status.to_string())
            })
            .collect();

        // Create ranking_result
        let result = sqlx::query_as::<_, RankingResult>(
            "INSERT INTO ranking_results (stage_id, total_participants, total_qualified)
             VALUES ($1, $2, $3) RETURNING *",
        )
        .bind(stage_id)
        .bind(total_participants)
        .bind(total_qualified)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        // Insert ranking_entries
        let mut entries = Vec::with_capacity(entries_with_status.len());
        for (idx, candidate, qual_status) in &entries_with_status {
            let entry = sqlx::query_as::<_, RankingEntry>(
                "INSERT INTO ranking_entries (ranking_result_id, participant_stage_id, rank, score, completion_time_secs, cheating_log_count, qualification_status)
                 VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
            )
            .bind(result.id)
            .bind(candidate.participant_stage_id)
            .bind((*idx as i32) + 1)
            .bind(candidate.score)
            .bind(candidate.completion_time_secs)
            .bind(candidate.cheating_log_count)
            .bind(qual_status)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;
            entries.push(entry);
        }

        // Update participant_stages.rank + status
        for (idx, candidate, qual_status) in &entries_with_status {
            let ps_status = match qual_status.as_str() {
                "qualified" => "ranked",
                "not_qualified" => "not_qualified",
                "disqualified" => "disqualified",
                _ => "ranked",
            };
            sqlx::query(
                "UPDATE participant_stages SET rank = $2, status = $3, updated_at = now() WHERE id = $1",
            )
            .bind(candidate.participant_stage_id)
            .bind((*idx as i32) + 1)
            .bind(ps_status)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;
        }

        Ok(RankingResultWithEntries { result, entries })
    }

    // ─── Result Lifecycle ───

    pub async fn get_latest_result(
        pool: &PgPool,
        stage_id: Uuid,
    ) -> Result<Option<RankingResult>, AppError> {
        sqlx::query_as::<_, RankingResult>(
            "SELECT * FROM ranking_results WHERE stage_id = $1 ORDER BY calculated_at DESC LIMIT 1",
        )
        .bind(stage_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn get_entries(
        pool: &PgPool,
        result_id: Uuid,
    ) -> Result<Vec<RankingEntry>, AppError> {
        sqlx::query_as::<_, RankingEntry>(
            "SELECT * FROM ranking_entries WHERE ranking_result_id = $1 ORDER BY rank",
        )
        .bind(result_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn transition_result(
        pool: &PgPool,
        result_id: Uuid,
        new_status: &str,
        actor_id: Option<Uuid>,
    ) -> Result<RankingResult, AppError> {
        let current = sqlx::query_as::<_, RankingResult>(
            "SELECT * FROM ranking_results WHERE id = $1",
        )
        .bind(result_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Ranking result not found".into()))?;

        // Validate transitions
        let valid = match (current.status.as_str(), new_status) {
            ("draft", "reviewed") => true,
            ("reviewed", "approved") => true,
            ("approved", "published") => true,
            _ => false,
        };
        if !valid {
            return Err(AppError::BadRequest(format!(
                "Cannot transition from '{}' to '{}'",
                current.status, new_status
            )));
        }

        let (approved_by, approved_at, published_at) = match new_status {
            "approved" => (actor_id, Some(chrono::Utc::now()), current.published_at),
            "published" => (current.approved_by, current.approved_at, Some(chrono::Utc::now())),
            _ => (current.approved_by, current.approved_at, current.published_at),
        };

        sqlx::query_as::<_, RankingResult>(
            "UPDATE ranking_results SET status = $2, approved_by = $3, approved_at = $4, published_at = $5
             WHERE id = $1 RETURNING *",
        )
        .bind(result_id)
        .bind(new_status)
        .bind(approved_by)
        .bind(approved_at)
        .bind(published_at)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)
    }

    // ─── Qualification / Promotion ───

    /// Promote qualified participants to next stage.
    /// Finds next stage by sequence + 1, creates participant_stages with status='registered'.
    pub async fn promote(
        pool: &PgPool,
        stage_id: Uuid,
    ) -> Result<PromotionResult, AppError> {
        // Get latest approved result
        let result = Self::get_latest_result(pool, stage_id)
            .await?
            .ok_or_else(|| AppError::NotFound("No ranking result for this stage".into()))?;

        if result.status != "approved" && result.status != "published" {
            return Err(AppError::BadRequest(
                "Ranking must be approved or published before promotion".into(),
            ));
        }

        // Find next stage
        let current_stage = sqlx::query_as::<_, (Uuid, Uuid, i32)>(
            "SELECT id, event_id, sequence FROM stages WHERE id = $1",
        )
        .bind(stage_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("Stage not found".into()))?;

        let next_stage = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM stages WHERE event_id = $1 AND sequence = $2",
        )
        .bind(current_stage.1)
        .bind(current_stage.2 + 1)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("No next stage found".into()))?;

        // Get qualified entries
        let qualified = sqlx::query_as::<_, RankingEntry>(
            "SELECT * FROM ranking_entries WHERE ranking_result_id = $1 AND qualification_status = 'qualified'",
        )
        .bind(result.id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        let mut promoted = 0i32;
        for entry in &qualified {
            // Get participant_id from participant_stages
            let participant_id = sqlx::query_scalar::<_, Uuid>(
                "SELECT participant_id FROM participant_stages WHERE id = $1",
            )
            .bind(entry.participant_stage_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

            // Create participant_stage for next stage (skip if exists)
            let inserted = sqlx::query(
                "INSERT INTO participant_stages (participant_id, stage_id, status)
                 VALUES ($1, $2, 'registered')
                 ON CONFLICT (participant_id, stage_id) DO NOTHING",
            )
            .bind(participant_id)
            .bind(next_stage)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

            if inserted.rows_affected() > 0 {
                promoted += 1;

                // Mark source as promoted
                sqlx::query(
                    "UPDATE participant_stages SET status = 'qualified', promoted_at = now(), updated_at = now() WHERE id = $1",
                )
                .bind(entry.participant_stage_id)
                .execute(pool)
                .await
                .map_err(AppError::Database)?;
            }
        }

        Ok(PromotionResult {
            stage_id,
            next_stage_id: next_stage,
            promoted_count: promoted,
        })
    }
}
