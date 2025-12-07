use std::sync::LazyLock;

use anyhow::Result;

use crate::{
    config::OmajinaiConfig,
    constants::RankedStatus,
    models::{Beatmap, BeatmapApiResponse},
};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub async fn fetch_beatmap(config: &OmajinaiConfig, beatmap_id: i32) -> Result<Vec<u8>> {
    let url = format!("{}/get-osu/{beatmap_id}", config.beatmap_service_url);

    let resp = CLIENT.get(&url).send().await?;
    let bytes = resp.bytes().await?;

    Ok(bytes.to_vec())
}

pub async fn api_get_beatmaps(
    api_key: &str,
    h: Option<&str>,
) -> Result<Option<Vec<BeatmapApiResponse>>> {
    let mut params = vec![];
    let url = if api_key.is_empty() {
        "https://osu.direct/api/get_beatmaps"
    } else {
        params.push(("k", api_key.into()));

        "https://old.ppy.sh/api/get_beatmaps"
    };

    if let Some(md5) = h {
        params.push(("h", md5.to_string()));
    }

    let response = CLIENT.get(url).query(&params).send().await?;

    if response.status() != 200 {
        return Ok(None);
    }

    let data: Vec<BeatmapApiResponse> = response.json().await?;

    if data.is_empty() {
        return Ok(None);
    }

    Ok(Some(data))
}

pub fn parse_beatmap_from_api(data: BeatmapApiResponse, direct: bool) -> Beatmap {
    let id = data.id.parse().unwrap_or(0);
    let set_id = data.set_id.parse().unwrap_or(0);
    let status = data.approved.parse().unwrap_or(0);
    let total_length = data.total_length.parse().unwrap_or(0);
    let max_combo = data.max_combo.and_then(|s| s.parse().ok()).unwrap_or(0);
    let bpm = data.bpm.and_then(|s| s.parse().ok()).unwrap_or(0.0);
    let cs = data.cs.parse().unwrap_or(0.0);
    let ar = data.ar.parse().unwrap_or(0.0);
    let od = data.od.parse().unwrap_or(0.0);
    let hp = data.hp.parse().unwrap_or(0.0);
    let diff = data.diff.parse().unwrap_or(0.0);

    let filename = format!(
        "{} - {} ({}) [{}].osu",
        data.artist, data.title, data.creator, data.version
    );

    let status = if direct {
        match status {
            0 => RankedStatus::Ranked.as_i32(),
            2 => RankedStatus::Pending.as_i32(),
            3 => RankedStatus::Qualified.as_i32(),
            5 => RankedStatus::Pending.as_i32(),
            7 => RankedStatus::Ranked.as_i32(),
            8 => RankedStatus::Loved.as_i32(),
            _ => RankedStatus::UpdateAvailable.as_i32(),
        }
    } else {
        match status {
            -2..=0 => RankedStatus::Pending.as_i32(),
            1 => RankedStatus::Ranked.as_i32(),
            2 => RankedStatus::Approved.as_i32(),
            3 => RankedStatus::Qualified.as_i32(),
            4 => RankedStatus::Loved.as_i32(),
            _ => RankedStatus::UpdateAvailable.as_i32(),
        }
    };

    let statuses = [
        RankedStatus::Ranked.as_i32(),
        RankedStatus::Approved.as_i32(),
        RankedStatus::Loved.as_i32(),
    ];
    let frozen = statuses.contains(&status);

    Beatmap {
        id,
        set_id,
        status,
        md5: data.md5,
        artist: data.artist,
        title: data.title,
        version: data.version,
        creator: data.creator,
        filename,
        last_update: chrono::Utc::now(),
        total_length,
        max_combo,
        frozen,
        plays: 0,
        passes: 0,
        mode: 0,
        bpm,
        cs,
        ar,
        od,
        hp,
        diff,
    }
}
