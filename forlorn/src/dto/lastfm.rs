use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetLastFm {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,

    #[allow(unused)]
    pub action: String,

    #[serde(rename = "b")]
    pub flag: String,
}
