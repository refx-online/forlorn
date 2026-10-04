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
         map_md5, map_status, score, xp_gained, pp, acc, max_combo, mods, n300, n100, n50, nmiss, ngeki, nkatu, 
         grade, status, mode, play_time, time_elapsed, client_flags, userid, perfect, online_checksum, 
         aim_assist_type, maple_values, aim_value, ar_value, aim, arc, cs, tw, twval, hdr, pinned, clock_rate
        ) values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(&beatmap.md5)
        .bind(beatmap.status)
        .bind(score.score)
        .bind(score.xp)
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
        .bind(score.aim_assist_type)
        .bind(&score.maple_values)
        .bind(score.aim_correction_value)
        .bind(score.ar_changer_value)
        .bind(score.uses_aim_correction)
        .bind(score.uses_ar_changer)
        .bind(score.uses_cs_changer)
        .bind(score.uses_timewarp)
        .bind(score.timewarp_value)
        .bind(score.uses_hd_remover)
        .bind(score.pinned)
        .bind(score.clock_rate)
        .execute(db.as_ref())
        .await?;

    Ok(res.last_insert_id())
}

/// Staff review queue: records why a score looked suspicious at submit
/// time (the score itself is still accepted). Seen in NERV flags.
pub async fn flag_score(
    db: &DbPoolManager,
    user_id: i32,
    score_id: u64,
    kind: &str,
    reason: &str,
    det: &str,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO scores_flag (user_id, score_id, kind, reason, det, created_at)
         VALUES (?, ?, ?, ?, ?, NOW())
         ON DUPLICATE KEY UPDATE reason = VALUES(reason), det = VALUES(det), created_at = NOW()",
    )
    .bind(user_id)
    .bind(score_id)
    .bind(kind)
    .bind(reason)
    .bind(det)
    .execute(db.as_ref())
    .await?;

    Ok(())
}

/// lazer-only score detail that the `scores` row cannot represent.
///
/// Nothing wrote this table before; the stable path can only carry the six legacy
/// hit counters, while lazer sends per-object statistics and a structured mod list.
/// `is_legacy_score` is 0 because lazer is by definition not a stable submission.
pub async fn insert_lazer_details(
    db: &DbPoolManager,
    score_id: u64,
    build_id: i64,
    started_at: chrono::DateTime<chrono::Utc>,
    ended_at: chrono::DateTime<chrono::Utc>,
    statistics: &serde_json::Value,
    mods: &[crate::routes::lazer::LazerMod],
) -> Result<()> {
    let mods_json = serde_json::to_value(mods)?;

    sqlx::query(
        "insert into lazer_scores (score_id, has_replay, started_at, ended_at, build_id, is_legacy_score, statistics_json, mods_json)
         values (?, 0, ?, ?, ?, 0, ?, ?)",
    )
    .bind(score_id)
    .bind(started_at.naive_utc())
    .bind(ended_at.naive_utc())
    .bind(build_id)
    .bind(statistics)
    .bind(mods_json)
    .execute(db.as_ref())
    .await?;

    Ok(())
}
