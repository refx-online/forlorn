use anyhow::Result;

use crate::{infrastructure::database::DbPoolManager, models::Clan};

pub async fn fetch_by_id(db: &DbPoolManager, clan_id: i32) -> Result<Option<Clan>> {
    let clan = sqlx::query_as::<_, Clan>("select * from clans where id = ?")
        .bind(clan_id)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(clan)
}
