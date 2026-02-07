use anyhow::Result;

use crate::{
    constants::SubmissionStatus,
    infrastructure::database::DbPoolManager,
    models::{Beatmap, Score},
};

pub async fn fetch_best(
    db: &DbPoolManager,
    user_id: i32,
    map_md5: &str,
    mode: i32,
) -> Result<Option<Score>> {
    let score = sqlx::query_as::<_, Score>(
        "select * from scores 
         where userid = ? AND map_md5 = ? AND mode = ? AND status = ?
         order by pp DESC
         limit 1",
    )
    .bind(user_id)
    .bind(map_md5)
    .bind(mode)
    .bind(SubmissionStatus::Best.as_i32())
    .fetch_optional(db.as_ref())
    .await?;

    Ok(score)
}

pub async fn fetch_by_id(db: &DbPoolManager, score_id: u64) -> Result<Option<Score>> {
    let score = sqlx::query_as::<_, Score>("select * from scores where id = ?")
        .bind(score_id)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(score)
}

pub async fn fetch_by_online_checksum(
    db: &DbPoolManager,
    checksum: &str,
) -> Result<Option<Score>, sqlx::Error> {
    sqlx::query_as::<_, Score>("select * from scores where online_checksum = ?")
        .bind(checksum)
        .fetch_optional(db.as_ref())
        .await
}

pub async fn update_status(db: &DbPoolManager, score_id: u64, status: i32) -> Result<()> {
    sqlx::query("update scores set status = ? where id = ?")
        .bind(status)
        .bind(score_id)
        .execute(db.as_ref())
        .await?;

    Ok(())
}

pub async fn update_preexisting_personal_best(db: &DbPoolManager, score: &Score) -> Result<()> {
    sqlx::query(
        "update scores set status = 1 
         where status = 2 and map_md5 = ?
         and userid = ? and mode = ?",
    )
    .bind(&score.map_md5)
    .bind(score.userid)
    .bind(score.mode)
    .execute(db.as_ref())
    .await?;

    Ok(())
}

pub async fn fetch_num_better_scores(db: &DbPoolManager, score: &Score) -> Result<u32> {
    // NOTE: only checks with pp instead of score.
    let num_better_scores: i64 = sqlx::query_scalar(
        "select count(*) from scores s 
         inner join users u on u.id = s.userid
         where s.map_md5 = ? and s.mode = ?
         and s.status = 2 and (u.priv & 1) != 0
         and s.pp > ?",
    )
    .bind(&score.map_md5)
    .bind(score.mode)
    .bind(score.pp)
    .fetch_one(db.as_ref())
    .await?;

    Ok((num_better_scores + 1) as u32)
}

pub async fn insert(db: &DbPoolManager, score: &Score, beatmap: &Beatmap) -> Result<u64> {
    let res = sqlx::query(
        "insert into scores (
         map_md5, map_status, score, pp, acc, max_combo, mods, n300, n100, n50, nmiss, ngeki, nkatu, 
         grade, status, mode, play_time, time_elapsed, client_flags, userid, perfect, online_checksum, 
        ) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(&beatmap.md5)
        .bind(beatmap.status)
        .bind(score.score)
        .bind(score.pp)
        .bind(score.acc)
        .bind(score.max_combo)
        .bind(score.mods)
        .bind(score.n300)
        .bind(score.n100)
        .bind(score.n50)
        .bind(score.nmiss)
        .bind(score.ngeki)
        .bind(score.nkatu)
        .bind(&score.grade)
        .bind(score.status)
        .bind(score.mode)
        .bind(score.play_time)
        .bind(score.time_elapsed)
        .bind(score.client_flags)
        .bind(score.userid)
        .bind(score.perfect)
        .bind(&score.online_checksum)
        .execute(db.as_ref())
        .await?;

    Ok(res.last_insert_id())
}
