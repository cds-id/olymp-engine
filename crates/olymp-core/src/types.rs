use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

// ─── Pagination ───

#[derive(Debug, Clone, Deserialize, Validate, ToSchema)]
pub struct PaginationParams {
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u32>,
    pub cursor: Option<String>,
}

impl PaginationParams {
    pub fn per_page_or_default(&self) -> u32 {
        self.per_page.unwrap_or(20)
    }
}

/// Standard offset-based pagination query params for list endpoints.
#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct ListParams {
    /// Page number (1-based, default: 1)
    #[param(minimum = 1)]
    pub page: Option<u32>,
    /// Items per page (default: 20, max: 100)
    #[param(minimum = 1, maximum = 100)]
    pub per_page: Option<u32>,
}

impl ListParams {
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> u32 {
        self.per_page.unwrap_or(20).min(100).max(1)
    }

    pub fn offset(&self) -> i64 {
        ((self.page() - 1) * self.per_page()) as i64
    }

    pub fn limit(&self) -> i64 {
        self.per_page() as i64
    }
}

// ─── Olympiad Tier ───

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    District,
    Province,
    National,
}

impl Tier {
    pub fn sequence(&self) -> i32 {
        match self {
            Tier::District => 1,
            Tier::Province => 2,
            Tier::National => 3,
        }
    }

    pub fn next(&self) -> Option<Tier> {
        match self {
            Tier::District => Some(Tier::Province),
            Tier::Province => Some(Tier::National),
            Tier::National => None,
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tier::District => write!(f, "district"),
            Tier::Province => write!(f, "province"),
            Tier::National => write!(f, "national"),
        }
    }
}

// ─── Stage Status ───

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    Draft,
    OpenRegistration,
    RegistrationClosed,
    Verification,
    ReadyForExam,
    ExamOpen,
    ExamClosed,
    Scoring,
    RankingReview,
    RankingApproved,
    ResultPublished,
    Promoted,
    Finalized,
    Cancelled,
}

impl std::fmt::Display for StageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        write!(f, "{}", s)
    }
}

// ─── Event Status ───

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Draft,
    Active,
    Finalized,
    Cancelled,
}

impl std::fmt::Display for EventStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventStatus::Draft => write!(f, "draft"),
            EventStatus::Active => write!(f, "active"),
            EventStatus::Finalized => write!(f, "finalized"),
            EventStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

// ─── Participant Stage Status ───

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ParticipantStageStatus {
    Registered,
    Verified,
    AssignedToExam,
    InProgress,
    Submitted,
    Scored,
    Ranked,
    Qualified,
    NotQualified,
    Disqualified,
    Winner,
    Finalist,
}

impl std::fmt::Display for ParticipantStageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        write!(f, "{}", s)
    }
}
