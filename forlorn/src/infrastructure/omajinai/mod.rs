use std::sync::LazyLock;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::OmajinaiConfig;

pub mod beatmap;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Serialize, Clone)]
pub struct PerformanceRequest {
    pub beatmap_id: i32,
    pub mode: i32,
    pub mods: i32,
    pub max_combo: i32,
    pub accuracy: f32,
    pub miss_count: i32,
    //pub lazer: Option<bool>,
    //pub passed_objects: Option<i32>,
    pub legacy_score: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PerformanceResult {
    pub stars: f32,
    pub pp: f32,
}

impl Default for PerformanceResult {
    fn default() -> Self {
        Self { stars: 0.0, pp: 0.0 }
    }
}

// matches {"data": {...}}
// TODO: remove
#[derive(Debug, Deserialize)]
struct Wrapper {
    data: PerformanceResult,
}

pub async fn calculate_pp(
    config: &OmajinaiConfig,
    perf_requests: &[PerformanceRequest],
) -> Result<Vec<PerformanceResult>> {
    let url = format!("{}/calculate", config.base_url);

    let mut out = Vec::with_capacity(perf_requests.len());

    for req in perf_requests {
        let mut req = req.clone();
        req.mode %= 4;

        let resp = CLIENT.get(&url)
            .query(&req)
            .send()
            .await?;

        let p: Wrapper = resp.json().await?;

        out.push(p.data);
    }

    Ok(out)
}
