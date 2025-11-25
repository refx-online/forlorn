use crate::infrastructure::database::DbPoolManager;
use crate::models::Score;
use anyhow::Result;

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
        .bind(score.aim_value)
        .bind(score.ar_value)
        .bind(score.aim)
        .bind(score.arc)
        .bind(score.cs)
        .bind(score.tw)
        .bind(score.twval)
        .bind(score.hdr)
        .bind(score.pinned)
        .execute(db.as_ref())
        .await?;

    Ok(())
}
