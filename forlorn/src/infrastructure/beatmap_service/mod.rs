use std::sync::LazyLock;

use anyhow::Result;
use serde::Deserialize;

use crate::models::Beatmap;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Debug, Clone)]
pub enum BeatmapLifecycleStatus {
    Unsubmitted,
    UpdateRequired,
    Available(Box<Beatmap>),
}

#[derive(Debug, Deserialize)]
struct BeatmapResponse {
    beatmap: Option<Beatmap>,
}

pub async fn fetch_beatmap_with_lifecycle(
    base_url: &str,
    md5: &str,
    filename: Option<&str>,
) -> Result<BeatmapLifecycleStatus> {
    let url = format!("{}/v1/beatmap/{}", base_url, md5);
    let mut req = CLIENT.get(&url);

    if let Some(f) = filename {
        req = req.query(&[("filename", f)]);
    }

    let resp = req.send().await?;
    let body = resp.text().await?;

    if body == "-1|false" {
        return Ok(BeatmapLifecycleStatus::Unsubmitted);
    }

    if body == "1|false" {
        return Ok(BeatmapLifecycleStatus::UpdateRequired);
    }

    let beatmap_resp: BeatmapResponse = serde_json::from_str(&body)?;

    if let Some(beatmap) = beatmap_resp.beatmap {
        Ok(BeatmapLifecycleStatus::Available(Box::new(beatmap)))
    } else {
        Ok(BeatmapLifecycleStatus::UpdateRequired)
    }
}
