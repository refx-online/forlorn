use anyhow::Result;

use crate::{
    constants::Mods,
    infrastructure::database::DbPoolManager,
    models::{Beatmap, Score, Stats},
    repository,
};

pub async fn recalculate(db: &DbPoolManager, stats: &mut Stats) -> Result<()> {
    let scores = repository::stats::fetch_total_scores(db, stats).await?;

    let mut total_acc = 0.0;
    let mut total_pp = 0.0;
    let mut last_idx = 0;

    for (idx, (acc, pp)) in scores.iter().enumerate() {
        let pp = *pp;
        let acc = *acc;

        total_pp += pp * (0.95_f32.powi(idx as i32));
        total_acc += acc * (0.95_f32.powi(idx as i32));
        last_idx = idx;
    }

    stats.acc =
        (total_acc * (100.0 / (20.0 * (1.0 - 0.95_f32.powi((last_idx + 1) as i32))))) / 100.0;

    stats.pp = (total_pp + calculate_bonus(db, stats).await?) as u32;

    Ok(())
}

pub async fn calculate_bonus(db: &DbPoolManager, stats: &Stats) -> Result<f32> {
    let result = repository::stats::fetch_bonus_count(db, stats).await?;

    let count = result.min(1000);
    let bonus_pp = 416.6667 * (1.0 - (0.995_f32.powi(count)));

    Ok(bonus_pp)
}

pub fn get_computed_playtime(score: &Score, beatmap: &Beatmap) -> u32 {
    if score.passed {
        beatmap.total_length as u32
    } else {
        let mut time_elapsed = score.time_elapsed as f32 / 1000.0;

        if score.mods().contains(Mods::DOUBLETIME) {
            time_elapsed /= 1.5;
        } else if score.mods().contains(Mods::HALFTIME) {
            time_elapsed /= 0.75;
        }

        if time_elapsed > beatmap.total_length as f32 {
            return 0;
        }

        time_elapsed as u32
    }
}
