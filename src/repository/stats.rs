use anyhow::Result;
use redis::AsyncCommands;

use crate::infrastructure::database::DbPoolManager;
use crate::infrastructure::redis::RedisConnectionManager;
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

pub async fn fetch_total_scores(
    db: &DbPoolManager,
    stats: &Stats
) -> Result<Vec<(f64, f64)>> {
    let scores = sqlx::query_as::<_, (f64, f64)>(
        r#"
        select s.acc, s.pp 
        from scores s 
        right join maps b on s.map_md5 = b.md5 
        where s.status = 2 and s.mode = ? and b.status in (2, 3) and s.userid = ? 
        order by s.pp desc 
        limit 100
        "#,
    )
    .bind(stats.mode)
    .bind(stats.id)
    .fetch_all(db.as_ref())
    .await?;

    Ok(scores)
}

pub async fn fetch_bonus_count(
    db: &DbPoolManager,
    stats: &Stats
) -> Result<i64> {
    let count = sqlx::query_scalar::<_, i64>(
        "select count(*) from scores s \
         right join maps b on s.map_md5 = b.md5 \
         where b.status in (2, 3) and s.status = 2 and s.mode = ? and s.userid = ?",
    )
    .bind(stats.mode)
    .bind(stats.id)
    .fetch_one(db.as_ref())
    .await?;

    Ok(count)
}

pub async fn get_global_rank(redis: &RedisConnectionManager, stats: &Stats) -> Result<usize> {
    let leaderboard = format!("bancho:leaderboard:{}", stats.mode);
    let mut conn = redis.lock().await;

    let rank: Option<u64> = conn
        .zrevrank::<_, _, Option<u64>>(&leaderboard, &stats.id.to_string())
        .await?;

    Ok(rank.map(|r| r as usize + 1).unwrap_or(0))
}

pub async fn get_country_rank(
    redis: &RedisConnectionManager,
    stats: &Stats,
    country: &str,
) -> Result<usize> {
    let leaderboard = format!("bancho:leaderboard:{}:{}", stats.mode, country);
    let mut conn = redis.lock().await;

    let rank: Option<u64> = conn
        .zrevrank::<_, _, Option<u64>>(&leaderboard, &stats.id.to_string())
        .await?;

    Ok(rank.map(|r| r as usize + 1).unwrap_or(0))
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
