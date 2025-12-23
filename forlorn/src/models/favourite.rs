use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Favourites {
    pub setid: u64,
    pub userid: i32,
    pub created_at: i32,
}
