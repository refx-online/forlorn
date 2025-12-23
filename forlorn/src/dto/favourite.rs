use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetFavourites {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,
}

#[derive(Debug, Deserialize)]
pub struct AddFavourites {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,

    #[serde(rename = "a")]
    pub mapset_id: i32,
}
