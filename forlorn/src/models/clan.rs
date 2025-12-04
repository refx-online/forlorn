use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Clan {
    pub id: i32,
    pub name: String,
    pub tag: String,
    pub owner: i32,
    pub created_at: DateTime<Utc>,
}
