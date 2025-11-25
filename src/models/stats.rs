use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::constants::GameMode;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Stats {
    pub id: i32,
    pub mode: i8,
    pub tscore: i32,
    pub rscore: i32,
    pub pp: i32,
    pub plays: i32,
    pub playtime: i32,
    pub acc: f32,
    pub max_combo: i32,
    pub total_hits: i32,
    pub replay_views: i32,
    pub xh_count: i32,
    pub x_count: i32,
    pub sh_count: i32,
    pub s_count: i32,
    pub a_count: i32,
    pub xp: i32,
}

impl Stats {
    pub fn mode(&self) -> GameMode {
        unsafe { std::mem::transmute(self.mode as u8) }
    }
}
