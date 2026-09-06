use super::Category;
use serde::{Deserialize, Serialize};

/// A reusable classification profile (row of the `profiles` table).
#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
}

/// A single app -> category rule belonging to a profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRule {
    pub id: i64,
    pub profile_id: i64,
    pub process_name: String,
    pub category: Category,
}

/// Input shape for `set_app_rules` (no id/profile_id, those are supplied
/// separately by the command).
#[derive(Debug, Clone, Deserialize)]
pub struct AppRuleInput {
    pub process_name: String,
    pub category: Category,
}

/// A row of the `usage_events` table (one contiguous active-window span).
#[derive(Debug, Clone, Serialize)]
pub struct UsageEvent {
    pub id: i64,
    pub session_id: i64,
    pub process_name: String,
    pub domain: Option<String>,
    pub category: Category,
    pub started_at: i64,
    pub ended_at: Option<i64>,
}
