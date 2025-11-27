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

#[derive(Default)]
pub struct SubmissionFields {
    pub exited_out: bool,
    pub fail_time: i32,
    pub visual_settings_b64: String,
    pub updated_beatmap_hash: String,
    pub storyboard_md5: Option<String>,
    pub iv_b64: Vec<u8>,
    pub unique_ids: String,
    pub score_time: i32,
    pub password_md5: String,
    pub osu_version: String,
    pub client_hash_b64: Vec<u8>,

    // refx original sin
    pub aim_value: i32,
    pub ar_value: f32,
    pub aim: bool,
    pub arc: bool,
    pub hdr: bool,
    pub cs: bool,
    pub tw: bool,
    pub twval: f32,
    pub refx: bool,
    pub score_data_b64: Vec<u8>,
    pub replay_file: Vec<u8>,
}
