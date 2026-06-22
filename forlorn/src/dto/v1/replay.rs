use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetReplay {
    #[serde(rename = "id")]
    pub score_id: u64,

    #[serde(rename = "include_headers", default = "default_include_headers")]
    pub include_headers: bool,
}

const fn default_include_headers() -> bool {
    true
}
