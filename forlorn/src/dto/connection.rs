use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetBanchoConnect {
    #[serde(rename = "u")]
    pub username: String,

    #[serde(rename = "h")]
    pub password_md5: String,

    #[allow(unused)]
    #[serde(rename = "v")]
    pub osu_ver: Option<String>,

    #[allow(unused)]
    #[serde(rename = "fail")]
    pub active_endpoint: Option<String>,

    #[allow(unused)]
    #[serde(rename = "fx")]
    pub net_framework_version: Option<String>,

    #[allow(unused)]
    #[serde(rename = "ch")]
    pub client_hash: Option<String>,

    #[allow(unused)]
    #[serde(rename = "retry")]
    pub retrying: Option<u8>,
}

impl GetBanchoConnect {
    #[allow(unused)]
    pub fn retrying(&self) -> bool {
        self.retrying.unwrap_or(0) > 0
    }
}
