use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetDirectSearch {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,

    #[serde(rename = "r")]
    pub status: i32,

    pub query: String,

    #[serde(rename = "m")]
    pub mode: i32,

    #[serde(rename = "p")]
    pub page: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetDirectSearchSet {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,

    #[serde(rename = "s")]
    pub map_set_id: Option<i32>,

    #[serde(rename = "b")]
    pub map_id: Option<i32>,

    #[serde(rename = "c")]
    pub map_md5: Option<String>,
}
