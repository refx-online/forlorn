use anyhow::Result;

use crate::constants::SubmissionStatus;
use crate::infrastructure::database::DbPoolManager;
use crate::models::Score;

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

pub async fn update_status(db: &DbPoolManager, score_id: i32, status: i32) -> Result<()> {
    sqlx::query("update scores set status = ? where id = ?")
        .bind(status)
        .bind(score_id)
        .execute(db.as_ref())
        .await?;

    Ok(())
}

pub async fn insert(db: &DbPoolManager, score: &Score) -> Result<()> {
    sqlx::query(
        "insert into scores (map_md5, score, xp_gained, pp, acc, max_combo, mods, n300, n100, n50, nmiss, ngeki, nkatu, grade, status, mode, play_time, time_elapsed, client_flags, userid, perfect, online_checksum, aim_value, ar_value, aim, arc, cs, tw, twval, hdr, pinned) \
         values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
        .bind(&score.map_md5)
        .bind(score.score)
        .bind(score.xp_gained)
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
        .bind(score.aim_correction_value)
        .bind(score.ar_changer_value)
        .bind(score.uses_aim_correction)
        .bind(score.uses_ar_changer)
        .bind(score.uses_cs_changer)
        .bind(score.uses_timewarp)
        .bind(score.timewarp_value)
        .bind(score.uses_hd_remover)
        .bind(score.pinned)
        .execute(db.as_ref())
        .await?;

    Ok(())
}
