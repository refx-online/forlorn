use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::config::OmajinaiConfig;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| reqwest::Client::new());

#[derive(Serialize)]
pub struct PerformanceRequest {
    pub beatmap_id: i32,
    pub mode: i32,
    pub mods: u32,
    pub max_combo: i32,
    pub accuracy: f32,
    pub miss_count: i32,
    pub lazer: Option<bool>,
    pub passed_objects: Option<i32>,
    pub legacy_score: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PerformanceResult {
    pub stars: f32,
    pub pp: f32,
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
        let mut params = serde_json::Map::new();

        if let serde_json::Value::Object(map) = serde_json::to_value(req)? {
            for (k, v) in map {
                if !v.is_null() {
                    params.insert(k, v);
                }
            }
        }

        if let Some(v) = params.get_mut("mode") {
            if let Some(n) = v.as_i64() {
                *v = serde_json::Value::from(n % 4);
            }
        }

        if req.lazer == Some(true) {
            params.insert(
                "mods".into(),
                serde_json::Value::String(req.mods.to_string()),
            );
        }

        let resp = CLIENT.get(&url).query(&params).send().await?;

        let p: Wrapper = resp.json().await?;

        out.push(p.data);
    }

    Ok(out)
}
