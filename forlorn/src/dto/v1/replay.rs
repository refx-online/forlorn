use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetReplay {
    #[serde(rename = "id")]
    pub score_id: u64,
}
