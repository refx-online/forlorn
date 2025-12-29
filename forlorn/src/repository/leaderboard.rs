use anyhow::Result;

use crate::{
    constants::LeaderboardType, infrastructure::database::DbPoolManager, models::LeaderboardScore,
};

fn cheat_columns(is_refx: bool) -> &'static str {
    if is_refx {
        ", s.aim_assist_type, s.maple_values, s.aim_value, s.ar_value, s.arc, s.cs, s.tw, s.twval, s.hdr, s.score"
    } else {
        // of course.
        ", NULL as aim_assist_type, \
           NULL as maple_values, \
           NULL as aim_value, \
           NULL as ar_value, \
           NULL as arc, \
           NULL as cs, \
           NULL as tw, \
           NULL as twval, \
           NULL as hdr, \
           NULL as score"
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn fetch_leaderboard_scores(
    db: &DbPoolManager,
    map_md5: &str,
    mode: i32,
    user_id: i32,
    leaderboard_type: LeaderboardType,
    mods: Option<i32>,
    country: Option<&str>,
    friend_ids: Option<&[i32]>,
    scoring_metric: &str,
    is_refx: bool,
) -> Result<Vec<LeaderboardScore>> {
    let cheat_val = cheat_columns(is_refx);

    let base_query = format!(
        "select s.id, s.{} as preferred_metric, \
         s.max_combo, s.n50, s.n100, s.n300, \
         s.nmiss, s.nkatu, s.ngeki, s.perfect, s.mods, \
         unix_timestamp(s.play_time) as play_time, u.id as userid, \
         coalesce(concat('[', c.tag, '] ', u.name), u.name) as name \
         {} \
         from scores s \
         inner join users u on u.id = s.userid \
         left join clans c on c.id = u.clan_id \
         where s.map_md5 = ? and s.status = 2 \
         and (u.priv & 1 or u.id = ?) and s.mode = ? \
         and s.online_checksum != 'lazer_score'",
        scoring_metric, cheat_val
    );

    let (query_with_filter, needs_friends_binding) = match leaderboard_type {
        LeaderboardType::Mods => (format!("{} and s.mods = ?", base_query), false),
        LeaderboardType::Friends => {
            if let Some(friends) = friend_ids {
                if friends.is_empty() {
                    (format!("{} and s.userid = ?", base_query), false)
                } else {
                    let placeholders = friends.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                    (
                        format!("{} and s.userid in ({})", base_query, placeholders),
                        true,
                    )
                }
            } else {
                (format!("{} and s.userid = ?", base_query), false)
            }
        },
        LeaderboardType::Country => (format!("{} and u.country = ?", base_query), false),
        _ => (base_query, false),
    };

    let final_query = format!(
        "{} order by preferred_metric desc limit 50",
        query_with_filter
    );

    let mut query = sqlx::query_as::<_, LeaderboardScore>(&final_query)
        .bind(map_md5)
        .bind(user_id)
        .bind(mode);

    match leaderboard_type {
        LeaderboardType::Mods => {
            query = query.bind(mods.unwrap_or(0));
        },
        LeaderboardType::Friends => {
            if needs_friends_binding {
                if let Some(friends) = friend_ids {
                    for friend_id in friends {
                        query = query.bind(friend_id);
                    }
                }
            } else {
                query = query.bind(user_id);
            }
        },
        LeaderboardType::Country => {
            query = query.bind(country.unwrap_or(""));
        },
        _ => {},
    };

    let scores = query.fetch_all(db.as_ref()).await?;

    Ok(scores)
}

pub async fn fetch_personal_best_score(
    db: &DbPoolManager,
    map_md5: &str,
    mode: i32,
    user_id: i32,
    scoring_metric: &str,
    is_refx: bool,
) -> Result<Option<LeaderboardScore>> {
    let cheat_val = cheat_columns(is_refx);

    let query = format!(
        "select s.id, s.{} as preferred_metric, \
         s.max_combo, s.n50, s.n100, s.n300, \
         s.nmiss, s.nkatu, s.ngeki, s.perfect, s.mods, \
         unix_timestamp(s.play_time) as play_time, u.id as userid, u.name as name \
         {} \
         from scores s \
         inner join users u on u.id = s.userid \
         where s.map_md5 = ? and s.mode = ? \
         and s.userid = ? and s.status = 2 \
         order by preferred_metric desc limit 1",
        scoring_metric, cheat_val
    );

    let pb = sqlx::query_as::<_, LeaderboardScore>(&query)
        .bind(map_md5)
        .bind(mode)
        .bind(user_id)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(pb)
}

pub async fn fetch_personal_best_rank(
    db: &DbPoolManager,
    map_md5: &str,
    mode: i32,
    score_value: f32,
    scoring_metric: &str,
) -> Result<i32> {
    let query = format!(
        "select count(*) from scores s \
         inner join users u on u.id = s.userid \
         where s.map_md5 = ? and s.mode = ? \
         and s.status = 2 and u.priv & 1 \
         and s.{} > ?",
        scoring_metric
    );

    let count: i64 = sqlx::query_scalar(&query)
        .bind(map_md5)
        .bind(mode)
        .bind(score_value)
        .fetch_one(db.as_ref())
        .await?;

    Ok((count + 1) as i32)
}
