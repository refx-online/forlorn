use anyhow::Result;

use crate::{
    infrastructure::database::DbPoolManager,
    models::{Condition, Score},
    repository,
};

pub async fn check_and_unlock_achievements(db: &DbPoolManager, score: &Score) -> Result<String> {
    let server_achievements = repository::achievement::fetch_all_achievements(db).await?;
    let player_achievement_ids =
        repository::user::fetch_user_achievements(db, score.userid).await?;

    let mut unlocked_achievements = Vec::new();

    for server_achievement in server_achievements {
        if player_achievement_ids.contains(&server_achievement.id) {
            continue;
        }

        let condition: Condition = match serde_json::from_value(server_achievement.cond.clone()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if condition.eval(score) {
            if (repository::user::create_user_achievement(db, score.userid, server_achievement.id)
                .await)
                .is_err()
            {
                continue;
            }

            unlocked_achievements.push(server_achievement);
        }
    }

    let achievements_str = unlocked_achievements
        .iter()
        .map(|a| format_achievement_string(&a.file, &a.name, &a.desc))
        .collect::<Vec<_>>()
        .join("/");

    Ok(achievements_str)
}

fn format_achievement_string(file: &str, name: &str, desc: &str) -> String {
    format!("{file}+{name}+{desc}")
}
