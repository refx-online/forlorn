use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetMarkChannelAsRead {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,

    #[serde(rename = "channel")]
    pub target: Option<String>,
}
