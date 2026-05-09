use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ─── DB Models ───

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RankingRule {
    pub id: Uuid,
    pub stage_id: Uuid,
    pub max_qualifiers: Option<i32>,
    pub min_score: Option<f64>,
    pub max_cheating_logs: Option<i32>,
    pub tiebreaker_order: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RankingResult {
    pub id: Uuid,
    pub stage_id: Uuid,
    pub calculated_at: DateTime<Utc>,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub total_participants: i32,
    pub total_qualified: i32,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, utoipa::ToSchema)]
pub struct RankingEntry {
    pub id: Uuid,
    pub ranking_result_id: Uuid,
    pub participant_stage_id: Uuid,
    pub rank: i32,
    pub score: f64,
    pub completion_time_secs: Option<i32>,
    pub cheating_log_count: i32,
    pub qualification_status: String,
    pub created_at: DateTime<Utc>,
}

// ─── Request DTOs ───

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateRankingRuleRequest {
    pub max_qualifiers: Option<i32>,
    pub min_score: Option<f64>,
    pub max_cheating_logs: Option<i32>,
    /// Tiebreaker strategy. Options: "score_desc", "time_asc", "cheating_asc"
    /// Examples:
    ///   tercepat:  ["time_asc","score_desc"]
    ///   terbesar:  ["score_desc","time_asc"]
    ///   keduanya:  ["score_desc","time_asc","cheating_asc"]
    pub tiebreaker_order: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateRankingRuleRequest {
    pub max_qualifiers: Option<i32>,
    pub min_score: Option<f64>,
    pub max_cheating_logs: Option<i32>,
    pub tiebreaker_order: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ReviewRequest {
    pub actor_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ApproveRequest {
    pub actor_id: Uuid,
}

// ─── Response DTOs ───

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RankingResultWithEntries {
    #[serde(flatten)]
    pub result: RankingResult,
    pub entries: Vec<RankingEntry>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PromotionResult {
    pub stage_id: Uuid,
    pub next_stage_id: Uuid,
    pub promoted_count: i32,
}

// ─── Tiebreaker logic ───

/// Sort key extracted from participant_stages for ranking
#[derive(Debug, Clone, FromRow)]
pub struct RankCandidate {
    pub participant_stage_id: Uuid,
    pub score: f64,
    pub completion_time_secs: Option<i32>,
    pub cheating_log_count: i32,
}

impl RankCandidate {
    /// Compare two candidates using tiebreaker order.
    /// Returns Ordering for sort (ascending = first wins).
    pub fn compare_by_tiebreakers(
        a: &RankCandidate,
        b: &RankCandidate,
        tiebreakers: &[String],
    ) -> std::cmp::Ordering {
        for tb in tiebreakers {
            let ord = match tb.as_str() {
                "score_desc" => b
                    .score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal),
                "time_asc" => {
                    let a_time = a.completion_time_secs.unwrap_or(i32::MAX);
                    let b_time = b.completion_time_secs.unwrap_or(i32::MAX);
                    a_time.cmp(&b_time)
                }
                "cheating_asc" => a.cheating_log_count.cmp(&b.cheating_log_count),
                _ => std::cmp::Ordering::Equal,
            };
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    }
}
