#![allow(unused)]

use axum::body::Bytes;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ScoreHeader {
    pub map_md5: String,
    pub username: String,
}

impl ScoreHeader {
    pub fn from_decrypted(score_data: &[String]) -> Option<Self> {
        if score_data.len() < 2 {
            return None;
        }

        Some(Self {
            map_md5: score_data[0].clone(),
            username: score_data[1].trim().to_string(),
        })
    }
}

pub struct ScoreSubmission {
    pub exited_out: Option<String>,
    pub fail_time: i32,
    pub visual_settings_b64: Option<String>,
    pub updated_beatmap_hash: Option<String>,
    pub storyboard_md5: Option<String>,
    pub iv_b64: Bytes,
    pub unique_ids: Option<String>,
    pub score_time: i32,
    pub password_md5: String,
    pub osu_version: String,
    pub client_hash_b64: Bytes,

    pub score_data_b64: Vec<u8>,
    pub replay_file: Vec<u8>,
}

impl ScoreSubmission {
    fn is_true(flag: &Option<String>) -> bool {
        flag.as_ref()
            .map(|s| s == "1" || s.to_lowercase() == "true")
            .unwrap_or(false)
    }

    pub fn exited_out(&self) -> bool {
        Self::is_true(&self.exited_out)
    }
}
