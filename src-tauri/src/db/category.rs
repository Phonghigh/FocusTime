use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

/// Classification bucket for an app or domain rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Work,
    Entertainment,
    Distraction,
}

impl Category {
    fn as_str(&self) -> &'static str {
        match self {
            Category::Work => "work",
            Category::Entertainment => "entertainment",
            Category::Distraction => "distraction",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "work" => Some(Category::Work),
            "entertainment" => Some(Category::Entertainment),
            "distraction" => Some(Category::Distraction),
            _ => None,
        }
    }
}

impl ToSql for Category {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl FromSql for Category {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let text = value.as_str()?;
        Category::from_str(text).ok_or(FromSqlError::InvalidType)
    }
}
