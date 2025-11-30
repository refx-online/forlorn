use anyhow::Result;

use crate::{infrastructure::database::DbPoolManager, models::Achievement};

pub async fn fetch_all_achievements(db: &DbPoolManager) -> Result<Vec<Achievement>> {
    let achievements =
        sqlx::query_as::<_, Achievement>("select id, file, name, `desc`, cond from achievements2")
            .fetch_all(db.as_ref())
            .await?;

    Ok(achievements)
}
