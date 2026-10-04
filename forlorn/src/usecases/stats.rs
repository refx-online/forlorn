use anyhow::Result;

use crate::{
    constants::{status::SubmissionStatus, Grade, Mods},
    infrastructure::{database::DbPoolManager, redis::RedisConnectionManager},
    models::{Beatmap, Score, Stats, User},
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

        if score.clock_rate != -1.0 {
            time_elapsed /= score.clock_rate as f32;
        } else if score.timewarp_value != -1.0 {
            time_elapsed /= score.timewarp_value / 100.0;
        } else if score.mods().contains(Mods::DOUBLETIME) || score.mods().contains(Mods::NIGHTCORE)
        {
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

/// The stats row after folding in a score, plus the row as it was beforehand
/// (`build_submission_charts` needs the delta to plot rank/pp movement).
pub struct AppliedStats {
    pub stats: Stats,
    pub prev_stats: Stats,
}

/// Fold one accepted score into the player's aggregate stats row and the redis
/// leaderboards.
///
/// Shared by the stable submission route and the lazer route so the two cannot
/// drift: both the `stats` table and the `bancho:leaderboard:*` ZSETs are
/// written here and nowhere else.
///
/// `prev_best` MUST be the player's previous best on this map+mode, read
/// *before* the new score was inserted. It drives the rscore delta and the
/// grade counter, and it has to be passed in rather than looked up here: once
/// the new row exists it is itself the best, so a lookup would always compare
/// the score against itself and award a delta of zero. `None` means the player
/// had no prior best on this map, which is a real, distinct case from "not
/// looked up".
pub async fn apply_score(
    db: &DbPoolManager,
    redis: &RedisConnectionManager,
    user: &User,
    score: &Score,
    beatmap: &Beatmap,
    prev_best: Option<&Score>,
) -> Result<AppliedStats> {
    let mut stats = repository::stats::fetch_by_user_mode(db, redis, user.id, score.mode)
        .await?
        .ok_or_else(|| anyhow::anyhow!("no stats row for user {} mode {}", user.id, score.mode))?;

    let prev_stats = stats.clone();
    let prev_pp = stats.pp as i32;

    stats.playtime += get_computed_playtime(score, beatmap);
    stats.plays += 1;
    stats.tscore += score.score as u64;
    stats.total_hits += score.total_hits();
    stats.xp += score.xp.round() as i32;

    if score.passed && beatmap.has_leaderboard_in(score.mode) {
        if score.max_combo as u32 > stats.max_combo {
            stats.max_combo = score.max_combo as u32;
        }

        if beatmap.awards_ranked_pp_in(score.mode)
            && score.status == SubmissionStatus::Best.as_i32()
        {
            let mut additional_rscore = score.score;
            if let Some(ref pb) = prev_best {
                additional_rscore -= pb.score;

                if score.grade() != pb.grade() {
                    if score.grade() >= Grade::A {
                        stats.increment_grade(score.grade());
                    }

                    if pb.grade() >= Grade::A {
                        stats.decrement_grade(pb.grade());
                    }
                }
            } else if score.grade() >= Grade::A {
                stats.increment_grade(score.grade());
            }

            stats.rscore += additional_rscore as u64;

            if score.pp > 0.0 {
                recalculate(db, &mut stats).await?;
            }

            let pp_gained = stats.pp as i32 - prev_pp;
            let mut message = format!("You achieved #{}!, ({:.2}pp)", score.rank, score.pp);

            if pp_gained > 0 {
                message += &format!(" and gained {pp_gained:.2}pp!");
            } else if pp_gained < 0 {
                message += &format!(" but lost {:.2}pp!", pp_gained.abs());
            }

            if let Ok(new_rank) =
                repository::stats::update_rank(redis, &stats, &user.country, user.restricted())
                    .await
            {
                stats.rank = new_rank;
            }

            let r = redis.clone();
            let uid = user.id;
            let text = message.clone();
            tokio::spawn(async move {
                let _ = crate::infrastructure::redis::publish::notify::notify(&r, uid, &text).await;
            });
        }
    }

    repository::stats::save(db, &stats).await?;

    // snapshot pp/rank history for graphs + peak rank. Fire and forget --
    // submission latency shouldn't depend on it.
    {
        let db = db.clone();
        let redis = redis.clone();
        let snapshot = stats.clone();
        let country = user.country.clone();
        tokio::spawn(async move {
            let country_rank =
                repository::stats::get_country_rank(&redis, &snapshot, &country)
                    .await
                    .unwrap_or(0);
            if let Err(e) = repository::history::capture(
                &db,
                snapshot.id,
                snapshot.mode as i32,
                snapshot.pp as i32,
                snapshot.rank,
                country_rank,
            )
            .await
            {
                tracing::warn!("history capture failed: {e:?}");
            }
        });
    }

    Ok(AppliedStats { stats, prev_stats })
}
