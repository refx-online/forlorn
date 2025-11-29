use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::constants::Grade;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Stats {
    pub id: i32,
    pub mode: i8,
    pub tscore: u64,
    pub rscore: u64,
    pub pp: u32,
    pub plays: u32,
    pub playtime: u32,
    pub acc: f32,
    pub max_combo: u32,
    pub total_hits: u32,
    pub replay_views: u32,
    pub xh_count: u32,
    pub x_count: u32,
    pub sh_count: u32,
    pub s_count: u32,
    pub a_count: u32,
    pub xp: i32,

    #[sqlx(skip)]
    pub rank: i32,
}

impl Stats {
    pub fn increment_grade(&mut self, grade: Grade) {
        match grade {
            Grade::XH => self.xh_count += 1,
            Grade::X => self.x_count += 1,
            Grade::SH => self.sh_count += 1,
            Grade::S => self.s_count += 1,
            Grade::A => self.a_count += 1,
            _ => {},
        }
    }

    pub fn decrement_grade(&mut self, grade: Grade) {
        match grade {
            Grade::XH => self.xh_count -= 1,
            Grade::X => self.x_count -= 1,
            Grade::SH => self.sh_count -= 1,
            Grade::S => self.s_count -= 1,
            Grade::A => self.a_count -= 1,
            _ => {},
        }
    }
}
