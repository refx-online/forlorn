use anyhow::Result;
use std::time::SystemTime;

use crate::infrastructure::database::DbPoolManager;
use crate::models::{Score, User};

pub async fn fetch_by_name(db: &DbPoolManager, username: &str) -> Result<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        "select id, name, safe_name, priv as privilege, pw_bcrypt, country, silence_end, donor_end, 
                creation_time, latest_activity, clan_id, clan_priv, preferred_mode, 
                play_style, custom_badge_name, custom_badge_icon, userpage_content, 
                api_key, whitelist from users where name = ?"
    )
        .bind(username)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(user)
}

pub async fn fetch_prev_n1(
    db: &DbPoolManager, 
    score: &Score
) -> sqlx::Result<Option<(i32, String)>> {
    let prev_n1 = 
    sqlx::query_as::<_, (i32, String)>(
    "select u.id, name from users u 
            inner join scores s on u.id = s.userid 
            where s.map_md5 = ? and s.mode = ? 
            and s.status = 2 and u.priv & 1 
            order by s.pp desc limit 1"
    )
        .bind(&score.map_md5)
        .bind(score.mode)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(prev_n1)
}

pub async fn update_latest_activity(db: &DbPoolManager, user_id: i32) -> Result<()> {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i32)
        .unwrap_or(0);

    sqlx::query("update users set latest_activity = ? where id = ?")
        .bind(time)
        .bind(user_id)
        .execute(db.as_ref())
        .await?;

    Ok(())
}
