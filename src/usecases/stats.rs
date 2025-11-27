use anyhow::Result;

use crate::infrastructure::database::DbPoolManager;
use crate::models::Stats;
use crate::repository;

pub async fn recalculate(db: &DbPoolManager, stats: &mut Stats) -> Result<()> {
    let scores = 
        repository::stats::fetch_total_scores(db, stats).await?;

    let mut total_acc = 0.0;
    let mut total_pp = 0.0;
    let mut last_idx = 0;

    for (idx, (acc, pp)) in scores.iter().enumerate() {
        let pp = *pp as f32;
        let acc = *acc as f32;

        total_pp += pp * (0.95_f32.powi(idx as i32));
        total_acc += acc * (0.95_f32.powi(idx as i32));
        last_idx = idx;
    }

    stats.acc =
        (total_acc * (100.0 / (20.0 * (1.0 - 0.95_f32.powi((last_idx + 1) as i32))))) / 100.0;

    stats.pp = (total_pp + calculate_bonus(db, stats).await?) as i32;

    Ok(())
}

pub async fn calculate_bonus(db: &DbPoolManager, stats: &Stats) -> Result<f32> {
    let result = 
        repository::stats::fetch_bonus_count(db, stats).await?;

    let count = result.min(1000) as f32;
    let bonus_pp = 416.6667 * (1.0 - (0.995_f32.powi(count as i32)));

    Ok(bonus_pp)
}

pub async fn save(db: &DbPoolManager, stats: &Stats) -> Result<()> {
    repository::stats::save(db, stats).await
}
