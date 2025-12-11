use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::user::User;
use crate::constants::{GameMode, Grade, Mods, SubmissionStatus};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Score {
    pub id: u64,
    pub map_md5: String,
    pub score: i32,
    #[sqlx(rename = "xp_gained")]
    pub xp: f32,
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

    // Somewhere in 2023,
    // I made a huge mistake of naming these fields poorly.
    // and i somehow think its a good idea to create
    // the "uses" fields instead of checking with the -1.0 values.
    // TODO: rename these fields in the database and remove "uses" fields later?
    #[sqlx(rename = "aim_value")]
    pub aim_correction_value: i32,
    #[sqlx(rename = "ar_value")]
    pub ar_changer_value: f32,
    #[sqlx(rename = "aim")]
    pub uses_aim_correction: bool,
    #[sqlx(rename = "arc")]
    pub uses_ar_changer: bool,
    #[sqlx(rename = "cs")]
    pub uses_cs_changer: bool,
    #[sqlx(rename = "tw")]
    pub uses_timewarp: bool,
    #[sqlx(rename = "twval")]
    pub timewarp_value: f32,
    #[sqlx(rename = "hdr")]
    pub uses_hd_remover: bool,

    pub pinned: bool,

    #[sqlx(skip)]
    pub rank: u32,
    #[sqlx(skip)]
    pub hypothetical_pp: f32,
    #[sqlx(skip)]
    pub stars: f32,
    #[sqlx(skip)]
    pub passed: bool,
}

impl Score {
    pub fn from_submission(data: &[String], map_md5: String, user_id: i32) -> Option<Self> {
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
            passed: data[12] == "True",
            mode: data[13].parse().ok()?,

            play_time: Utc.from_utc_datetime(&play_time),

            client_flags: data[15].chars().filter(|&c| c == ' ').count() as i32 & !4,

            map_md5,
            userid: user_id,

            // will set later
            id: 0,
            xp: 0.0,
            pp: 0.0,
            acc: 0.0,
            status: 0,
            time_elapsed: 0,
            aim_correction_value: 0,
            ar_changer_value: 0.0,
            timewarp_value: 0.0,
            uses_aim_correction: false,
            uses_ar_changer: false,
            uses_cs_changer: false,
            uses_timewarp: false,
            uses_hd_remover: false,
            pinned: false,
            rank: 0,
            hypothetical_pp: 0.0,
            stars: 0.0,
        })
    }

    pub fn mode(&self) -> GameMode {
        GameMode::from_params(self.mode, self.mods())
    }

    pub fn mods(&self) -> Mods {
        Mods::from_bits_truncate(self.mods)
    }

    pub fn grade(&self) -> Grade {
        Grade::from_str(&self.grade)
    }

    pub fn status(&self) -> SubmissionStatus {
        SubmissionStatus::from_i32(self.status)
    }

    pub fn get_ach_stat(&self, name: &str) -> f64 {
        match name {
            "accuracy" => self.acc as f64,
            "sr" => self.stars as f64,
            "mods" => self.mods as f64,
            "mode_vn" => self.mode().as_vanilla() as f64,
            "combo" => self.max_combo as f64,
            "max_combo" => self.max_combo as f64,
            "perfect" => {
                if self.perfect {
                    1.0
                } else {
                    0.0
                }
            },
            "300" => self.n300 as f64,
            "100" => self.n100 as f64,
            "50" => self.n50 as f64,
            "miss" => self.nmiss as f64,
            "geki" => self.ngeki as f64,
            "katu" => self.nkatu as f64,
            _ => 0.0,
        }
    }

    pub fn check_pp_cap(&self, user: &User) -> (bool, Option<i32>) {
        // does not have to restrict again if the user is already restricted.
        if user.restricted() {
            return (false, None);
        }

        let pp_caps = self.mode().pp_cap();
        if pp_caps.is_empty() {
            return (false, None);
        }

        let threshold = pp_caps[user.whitelist_stage().min(pp_caps.len() - 1)];

        (self.pp > threshold as f32, Some(threshold))
    }

    pub fn total_hits(&self) -> u32 {
        let mut total_hits = self.n300 as u32 + self.n100 as u32 + self.n50 as u32;

        if self.mode().ngeki_nkatu() {
            total_hits += self.ngeki as u32 + self.nkatu as u32;
        }

        total_hits
    }
}
