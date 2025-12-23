use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetFriends {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,
}
