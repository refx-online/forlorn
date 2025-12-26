use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetLastFm {
    #[serde(rename = "us")]
    pub username: String,

    #[serde(rename = "ha")]
    pub password_md5: String,

    #[allow(unused)]
    pub action: String,

    #[serde(rename = "b")]
    pub flag: String, // or beatmap id
}
