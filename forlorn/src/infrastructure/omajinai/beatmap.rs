use std::{sync::LazyLock, time::Duration};

use anyhow::Result;
use tokio::time;

use crate::config::OmajinaiConfig;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub async fn fetch_beatmap(config: &OmajinaiConfig, beatmap_id: i32) -> Result<Vec<u8>> {
    let url = format!("{}/get-osu/{beatmap_id}", config.beatmap_service_url);

    for att in 0..3 {
        let resp = CLIENT.get(&url).send().await?;

        let bytes = resp.bytes().await?;

        if bytes.starts_with(b"osu file format") {
            return Ok(bytes.to_vec());
        }

        // beatmap service/my wifi is too slow!, retry.
        if att < 2 {
            time::sleep(Duration::from_millis(100 * (1 << att))).await;
        }
    }

    unreachable!()
}
