use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::constants::Grade;

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
    pub xp: i32, // unused

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
            Grade::XH => self.xh_count = (self.xh_count - 1).max(0),
            Grade::X => self.x_count = (self.x_count - 1).max(0),
            Grade::SH => self.sh_count = (self.sh_count - 1).max(0),
            Grade::S => self.s_count = (self.s_count - 1).max(0),
            Grade::A => self.a_count = (self.a_count - 1).max(0),
            _ => {},
        }
    }
}
