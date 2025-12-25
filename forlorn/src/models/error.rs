use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dto::error::GetError;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClientError {
    pub id: i64,

    pub user_id: i32,
    pub username: String,

    pub feedback: Option<String>,
    pub exception: Option<String>,
    pub stacktrace: Option<String>,

    pub created_at: DateTime<Utc>,
}

impl ClientError {
    pub fn from_error(error: GetError) -> Self {
        Self {
            id: 0,
            user_id: error.user_id.unwrap_or(0),
            username: "Offline user".to_string(),
            feedback: error.feedback,
            exception: error.exception,
            stacktrace: error.stacktrace.filter(|s| s.len() < 2000),
            created_at: Utc::now(),
        }
    }
}
