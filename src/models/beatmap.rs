use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Beatmap {
    pub server: String,
    pub id: i32,
    pub set_id: i32,
    pub status: i32,
    pub md5: String,
    pub artist: String,
    pub title: String,
    pub version: String,
    pub creator: String,
    pub filename: String,
    pub last_update: DateTime<Utc>,
    pub total_length: i32,
    pub max_combo: i32,
    pub frozen: bool,
    pub plays: i32,
    pub passes: i32,
    pub mode: i8,
    pub bpm: f32,
    pub cs: f32,
    pub ar: f32,
    pub od: f32,
    pub hp: f32,
    pub diff: f32,
}
