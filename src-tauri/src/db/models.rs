use serde::Serialize;

/// A reusable classification profile (row of the `profiles` table).
#[derive(Debug, Clone, Serialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
}
