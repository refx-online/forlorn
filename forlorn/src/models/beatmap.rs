use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::constants::RankedStatus;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Beatmap {
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

#[derive(Debug, Deserialize, Clone)]
pub struct BeatmapApiResponse {
    #[serde(rename = "beatmap_id")]
    pub id: String,
    #[serde(rename = "beatmapset_id")]
    pub set_id: String,
    #[serde(rename = "file_md5")]
    pub md5: String,
    pub artist: String,
    pub title: String,
    pub version: String,
    pub creator: String,
    pub approved: String,
    #[serde(rename = "total_length")]
    pub total_length: String,
    #[serde(rename = "max_combo")]
    pub max_combo: Option<String>,
    pub bpm: Option<String>,
    #[serde(rename = "diff_size")]
    pub cs: String,
    #[serde(rename = "diff_approach")]
    pub ar: String,
    #[serde(rename = "diff_overall")]
    pub od: String,
    #[serde(rename = "diff_drain")]
    pub hp: String,
    #[serde(rename = "difficultyrating")]
    pub diff: String,
}

#[derive(Debug, Deserialize)]
pub struct BeatmapChild {
    #[serde(rename = "BeatmapID")]
    pub beatmap_id: i32,
    #[serde(rename = "FileMD5")]
    pub file_md5: String,
    #[serde(rename = "DiffName")]
    pub diff_name: String,
    #[serde(rename = "TotalLength")]
    pub total_length: i32,
    #[serde(rename = "MaxCombo")]
    pub max_combo: i32,
    #[serde(rename = "Playcount")]
    pub playcount: i32,
    #[serde(rename = "Passcount")]
    pub passcount: i32,
    #[serde(rename = "Mode")]
    pub mode: i8,
    #[serde(rename = "BPM")]
    pub bpm: f32,
    #[serde(rename = "CS")]
    pub cs: f32,
    #[serde(rename = "AR")]
    pub ar: f32,
    #[serde(rename = "OD")]
    pub od: f32,
    #[serde(rename = "HP")]
    pub hp: f32,
    #[serde(rename = "DifficultyRating")]
    pub difficulty_rating: f32,
}

#[derive(Debug, Deserialize)]
pub struct BeatmapSet {
    #[serde(rename = "SetID")]
    pub set_id: i32,
    #[serde(rename = "RankedStatus")]
    pub ranked_status: i32,
    #[serde(rename = "Artist")]
    pub artist: String,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Creator")]
    pub creator: String,
    #[serde(rename = "LastUpdate")]
    pub last_update: String,
    #[serde(rename = "ChildrenBeatmaps")]
    pub children_beatmaps: Vec<BeatmapChild>,
}

#[derive(sqlx::FromRow, Clone)]
pub struct BeatmapSetInfo {
    pub set_id: i32,
    pub artist: String,
    pub title: String,
    pub status: i32,
    pub creator: String,
    pub last_update: DateTime<Utc>,
}

impl Beatmap {
    pub fn full_name(&self) -> String {
        format!("{} - {} [{}]", self.artist, self.title, self.version)
    }

    pub fn url(&self) -> String {
        // todo: use env
        format!("https://remeliah.cyou/beatmaps/{}", self.id)
    }

    pub fn has_leaderboard(&self) -> bool {
        [
            RankedStatus::Qualified.as_i32(),
            RankedStatus::Ranked.as_i32(),
            RankedStatus::Approved.as_i32(),
            RankedStatus::Loved.as_i32(),
        ]
        .contains(&self.status)
    }

    pub fn awards_ranked_pp(&self) -> bool {
        [RankedStatus::Ranked.as_i32(), RankedStatus::Approved.as_i32()].contains(&self.status)
    }
}
