use std::sync::LazyLock;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    config::OmajinaiConfig,
    constants::{GameMode, Mods},
};

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

    /// hypothetical, as it removes misses from the score
    /// and calculate the performance point and it results this.
    pub hypothetical_pp: f32,
}

impl Default for PerformanceResult {
    fn default() -> Self {
        Self {
            stars: 0.0,
            pp: 0.0,
            hypothetical_pp: 0.0,
        }
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
    requests: &PerformanceRequest,
) -> Result<PerformanceResult> {
    let url = format!("{}/calculate", config.base_url);

    let mut performance_request = requests.clone();
    let mut mods = Mods::from_bits_truncate(performance_request.mods);

    if (performance_request.mode == GameMode::CHEAT_OSU.as_i32()
        || performance_request.mode == GameMode::CHEAT_CHEAT_OSU.as_i32())
        && mods.contains(Mods::RELAX)
    {
        // NOTE: on the client, it has 2 relaxes. relax mod and relax "cheat".
        //       we should not calculate relax mods on that client because its nerf was deemed "too harsh"
        //       and because its a cheating stuff, and we know how the people that plays it reeaallly wants
        mods.remove(Mods::RELAX);
    }

    if performance_request.mode == GameMode::CHEAT_OSU.as_i32() {
        // since streams are too stupid, we'll use "relax" nerfs to combat that
        mods.insert(Mods::RELAX);
    }

    performance_request.mods = mods.bits();
    // mode as vanilla
    performance_request.mode %= 4;

    let resp = CLIENT.get(&url).query(&performance_request).send().await?;

    let p: Wrapper = resp.json().await?;

    Ok(p.data)
}
