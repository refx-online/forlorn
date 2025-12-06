use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetReplay {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,

    #[allow(unused)]
    #[serde(rename = "m")]
    pub mode: i32,

    #[serde(rename = "c")]
    pub score_id: u64,
}
