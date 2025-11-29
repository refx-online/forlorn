use anyhow::Result;
use std::sync::LazyLock;

use crate::config::OmajinaiConfig;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub async fn fetch_beatmap(config: &OmajinaiConfig, beatmap_id: i32) -> Result<Vec<u8>> {
    let url = format!("{}/get-osu/{beatmap_id}", config.beatmap_service_url);

    let resp = CLIENT.get(&url).send().await?;
    let bytes = resp.bytes().await?;

    Ok(bytes.to_vec())
}
