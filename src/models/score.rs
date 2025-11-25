use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::constants::{GameMode, Mods};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Score {
    pub id: i32,
    pub map_md5: String,
    pub score: i32,
    pub xp_gained: i32,
    pub pp: f32,
    pub acc: f32,
    pub max_combo: i32,
    pub mods: i32,
    pub n300: i32,
    pub n100: i32,
    pub n50: i32,
    pub nmiss: i32,
    pub ngeki: i32,
    pub nkatu: i32,
    pub grade: String,
    pub status: i32,
    pub mode: i32,
    pub play_time: DateTime<Utc>,
    pub time_elapsed: i32,
    pub client_flags: i32,
    pub userid: i32,
    pub perfect: bool,
    pub online_checksum: String,
    pub aim_value: i32,
    pub ar_value: f32,
    pub aim: bool,
    pub arc: bool,
    pub cs: bool,
    pub tw: f32,
    pub twval: bool,
    pub hdr: bool,
    pub pinned: bool,
}

impl Score {
    pub fn from_submission(data: &[String]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }

        let play_time = NaiveDateTime::parse_from_str(&data[14], "%y%m%d%H%M%S").ok()?;

        Some(Self {
            online_checksum: data[0].clone(),
            n300: data[1].parse().ok()?,
            n100: data[2].parse().ok()?,
            n50: data[3].parse().ok()?,
            ngeki: data[4].parse().ok()?,
            nkatu: data[5].parse().ok()?,
            nmiss: data[6].parse().ok()?,
            score: data[7].parse().ok()?,
            max_combo: data[8].parse().ok()?,
            perfect: data[9] == "True",
            grade: data[10].clone(),
            mods: data[11].parse().ok()?,
            mode: data[13].parse().ok()?,

            #[allow(deprecated)]
            play_time: DateTime::from_utc(play_time, Utc),

            client_flags: data[15].chars().filter(|&c| c == ' ').count() as i32 & !4,

            id: 0,
            map_md5: String::new(),
            xp_gained: 0,
            pp: 0.0,
            acc: 0.0,
            status: 0,
            time_elapsed: 0,
            userid: 0,
            aim_value: 0,
            ar_value: 0.0,
            aim: false,
            arc: false,
            cs: false,
            tw: 0.0,
            twval: false,
            hdr: false,
            pinned: false,
        })
    }

    pub fn mode(&self) -> GameMode {
        GameMode::from_params(self.mode as u8, self.mods())
    }

    pub fn mods(&self) -> Mods {
        Mods::from_bits_truncate(self.mods as u32)
    }
}
