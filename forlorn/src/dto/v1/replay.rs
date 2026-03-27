use serde::Deserialize;

#[derive(Deserialize)]
#[allow(unused)]
pub struct GetReplay {
    #[serde(rename = "id")]
    pub score_id: i32,
}
