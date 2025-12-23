use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetBeatmapInfo {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,

    #[serde(rename = "Filenames")]
    pub filenames: Vec<String>,

    #[serde(rename = "Ids")]
    pub ids: Vec<u64>,
}
