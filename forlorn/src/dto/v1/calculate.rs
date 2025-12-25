use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetCalculateMap {
    #[serde(rename = "md5")]
    pub map_id: i32,

    #[serde(rename = "mode")]
    pub mode: Option<i32>,

    #[serde(rename = "mods")]
    pub mods: Option<i32>,

    #[serde(rename = "acc")]
    pub accuracy: Option<f32>,

    #[serde(rename = "miss")]
    pub misses: Option<i32>,

    #[serde(rename = "combo")]
    pub max_combo: Option<i32>,

    #[serde(rename = "score")]
    pub legacy_score: Option<i32>,
}
