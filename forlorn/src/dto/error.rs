use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetError {
    // NOTE: we only depend on user_id
    #[allow(unused)]
    #[serde(rename = "u")]
    pub username: Option<String>,

    #[allow(unused)]
    #[serde(rename = "h")]
    pub password_md5: Option<String>,

    #[serde(rename = "i")]
    pub user_id: Option<i32>,

    pub stacktrace: Option<String>,

    pub exception: Option<String>,

    pub feedback: Option<String>,

    #[allow(unused)]
    pub config: String,

    #[allow(unused)]
    #[serde(rename = "exehash")]
    pub exe_hash: String,

    #[allow(unused)]
    pub version: String,

    #[serde(rename = "ss")]
    pub screenshot_data: Option<Vec<u8>>,
}
