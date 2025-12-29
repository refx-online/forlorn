use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::Json};

use crate::models::{AimAssistType, MapleAimAssistValues};

#[derive(Serialize, Deserialize, FromRow)]
pub struct LeaderboardScore {
    pub id: u64,
    pub preferred_metric: f32,
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

    pub aim_assist_type: Option<i8>,
    pub maple_values: Option<Json<MapleAimAssistValues>>,
    #[sqlx(rename = "aim_value")]
    pub aim_correction_value: Option<i32>,
    #[sqlx(rename = "ar_value")]
    pub ar_changer_value: Option<f32>,
    #[sqlx(rename = "arc")]
    pub uses_ar_changer: Option<bool>,
    #[sqlx(rename = "cs")]
    pub uses_cs_changer: Option<bool>,
    #[sqlx(rename = "tw")]
    pub uses_timewarp: Option<bool>,
    #[sqlx(rename = "twval")]
    pub timewarp_value: Option<f32>,
    #[sqlx(rename = "hdr")]
    pub uses_hd_remover: Option<bool>,
    #[sqlx(rename = "score")]
    // because of legacy score calculation, we need an actual score to calculate the misses.
    pub actual_score: Option<i32>,
}

impl LeaderboardScore {
    pub fn aim_assist_type(&self) -> AimAssistType {
        self.aim_assist_type
            .map(AimAssistType::from_i8)
            .unwrap_or_else(|| AimAssistType::None)
    }
}

pub struct PersonalBest {
    pub score: LeaderboardScore,
    pub rank: i32,
}
