use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetRating {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,

    #[allow(unused)]
    #[serde(rename = "c")]
    pub map_md5: String,

    #[serde(rename = "v")]
    pub rating: Option<u8>,
}
