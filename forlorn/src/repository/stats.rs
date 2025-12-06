use anyhow::Result;
use redis::AsyncCommands;

use crate::{
    infrastructure::{database::DbPoolManager, redis::RedisConnectionManager},
    models::Stats,
};

pub async fn fetch_by_user_mode(
    db: &DbPoolManager,
    redis: &RedisConnectionManager,
    userid: i32,
    mode: i32,
) -> Result<Option<Stats>> {
    let mut stats = match sqlx::query_as::<_, Stats>(
        "select id, mode, tscore, rscore, pp, plays, playtime, acc, max_combo, total_hits, replay_views, xh_count, x_count, sh_count, s_count, a_count, xp \
         from stats where id = ? and mode = ?"
    )
    .bind(userid)
    .bind(mode)
    .fetch_optional(db.as_ref())
    .await? {
        Some(stats) => stats,
        None => return Ok(None),
    };

    stats.rank = get_global_rank(redis, &stats).await.unwrap_or(0);

    Ok(Some(stats))
}

pub async fn fetch_total_scores(db: &DbPoolManager, stats: &Stats) -> Result<Vec<(f32, f32)>> {
    let scores = sqlx::query_as::<_, (f32, f32)>(
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

pub async fn fetch_bonus_count(db: &DbPoolManager, stats: &Stats) -> Result<i32> {
    let count = sqlx::query_scalar::<_, i32>(
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

pub async fn get_global_rank(redis: &RedisConnectionManager, stats: &Stats) -> Result<i32> {
    let leaderboard = format!("bancho:leaderboard:{}", stats.mode);
    let mut conn = redis.lock().await;

    let rank: Option<i32> = conn
        .zrevrank::<_, _, Option<i32>>(&leaderboard, &stats.id.to_string())
        .await?;

    Ok(rank.map(|r| r + 1).unwrap_or(0))
}

pub async fn update_rank(
    redis: &RedisConnectionManager,
    stats: &Stats,
    country: &str,
    is_restricted: bool,
) -> Result<i32> {
    if !is_restricted {
        let mut conn = redis.lock().await;

        let global_leaderboard = format!("bancho:leaderboard:{}", stats.mode);
        conn.zadd::<_, _, _, ()>(&global_leaderboard, stats.id.to_string(), stats.pp)
            .await?;

        let country_leaderboard = format!("bancho:leaderboard:{}:{}", stats.mode, country);
        conn.zadd::<_, _, _, ()>(&country_leaderboard, stats.id.to_string(), stats.pp)
            .await?;
    }

    get_global_rank(redis, stats).await
}

pub async fn increment_replay_views(db: DbPoolManager, user_id: i32, mode: i32) -> Result<()> {
    sqlx::query("update stats set replay_views = replay_views + 1 where id = ? and mode = ?")
        .bind(user_id)
        .bind(mode)
        .execute(db.as_ref())
        .await?;

    Ok(())
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
