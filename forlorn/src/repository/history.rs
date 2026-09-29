use anyhow::Result;

use crate::infrastructure::database::DbPoolManager;

// NOTE: user_profile_history had writers nowhere — mist only reads it, so
// peak rank was always empty. capture() snapshots (pp, rank, country_rank)
// after stats updates, but only when useful: first capture, older than a
// day, a new pp best, or a new rank best. keeps the table small and the
// pp/rank graphs meaningful.
pub async fn capture(
    db: &DbPoolManager,
    user_id: i32,
    mode: i32,
    pp: i32,
    rank: i32,
    country_rank: i32,
) -> Result<()> {
    let (max_pp, min_rank, fresh): (i64, i64, i8) = sqlx::query_as(
        "SELECT COALESCE(MAX(pp), -1),
                COALESCE(MIN(CASE WHEN `rank` > 0 THEN `rank` END), 2147483647),
                COALESCE(MAX(captured_at > NOW() - INTERVAL 1 DAY), 0)
         FROM user_profile_history
         WHERE user_id = ? AND mode = ?",
    )
    .bind(user_id)
    .bind(mode)
    .fetch_one(db.as_ref())
    .await?;

    let worth_it = fresh == 0 || pp as i64 > max_pp || (rank > 0 && (rank as i64) < min_rank);
    if !worth_it {
        return Ok(());
    }

    sqlx::query(
        "INSERT INTO user_profile_history (user_id, mode, pp, `rank`, country_rank)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(mode)
    .bind(pp)
    .bind(rank)
    .bind(country_rank)
    .execute(db.as_ref())
    .await?;

    Ok(())
}
