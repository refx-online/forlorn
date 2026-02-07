use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct LeaderboardScore {
    pub id: u64,
    pub pp: f32,
    pub max_combo: i32,
    pub n50: i32,
    pub n100: i32,
    pub n300: i32,
    pub nmiss: i32,
    pub nkatu: i32,
    pub ngeki: i32,
    pub perfect: bool,
    pub mods: i32,
    pub play_time: i64,
    pub userid: i32,
    pub name: String,
}

pub struct PersonalBest {
    pub score: LeaderboardScore,
    pub rank: i32,
}
