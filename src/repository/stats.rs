use anyhow::Result;

use crate::infrastructure::database::DbPoolManager;
use crate::models::Stats;

pub async fn fetch_by_user_mode(
    db: &DbPoolManager,
    userid: i32,
    mode: i8,
) -> Result<Option<Stats>> {
    let stats = sqlx::query_as::<_, Stats>(
        "select id, mode, tscore, rscore, pp, plays, playtime, acc, max_combo, total_hits, replay_views, xh_count, x_count, sh_count, s_count, a_count, xp \
         from stats where id = ? and mode = ?"
    )
    .bind(userid)
    .bind(mode)
    .fetch_optional(db.as_ref())
    .await?;

    Ok(stats)
}

pub async fn save(db: &DbPoolManager, stats: &Stats) -> Result<()> {
    sqlx::query(
        "update stats set tscore = ?, rscore = ?, pp = ?, plays = ?, playtime = ?, acc = ?, max_combo = ?, total_hits = ?, replay_views = ?, xh_count = ?, x_count = ?, sh_count = ?, s_count = ?, a_count = ?, xp = ? where id = ? and mode = ?"
    )
    .bind(stats.tscore)
    .bind(stats.rscore)
    .bind(stats.pp)
    .bind(stats.plays)
    .bind(stats.playtime)
    .bind(stats.acc)
    .bind(stats.max_combo)
    .bind(stats.total_hits)
    .bind(stats.replay_views)
    .bind(stats.xh_count)
    .bind(stats.x_count)
    .bind(stats.sh_count)
    .bind(stats.s_count)
    .bind(stats.a_count)
    .bind(stats.xp)
    .bind(stats.id)
    .bind(stats.mode)
    .execute(db.as_ref())
    .await?;

    Ok(())
}
