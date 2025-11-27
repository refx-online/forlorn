use axum::body::Bytes;

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

    // refx original sin
    pub aim_value: i32,
    pub ar_value: f32,
    pub aim: Option<String>,
    pub arc: Option<String>,
    pub hdr: Option<String>,
    pub cs: Option<String>,
    pub tw: Option<String>,
    pub twval: f32,
    pub refx: Option<String>,
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

    pub fn aim(&self) -> bool {
        Self::is_true(&self.aim)
    }

    pub fn arc(&self) -> bool {
        Self::is_true(&self.arc)
    }

    pub fn hdr(&self) -> bool {
        Self::is_true(&self.hdr)
    }

    pub fn cs(&self) -> bool {
        Self::is_true(&self.cs)
    }

    pub fn tw(&self) -> bool {
        Self::is_true(&self.tw)
    }

    pub fn refx(&self) -> bool {
        Self::is_true(&self.refx)
    }
}
