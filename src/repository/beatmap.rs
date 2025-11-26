use anyhow::Result;
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::sync::RwLock;

use crate::infrastructure::database::DbPoolManager;
use crate::models::Beatmap;

static BEATMAP_CACHE: LazyLock<RwLock<HashMap<String, Beatmap>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub async fn fetch_by_md5(db: &DbPoolManager, md5: &str) -> Result<Option<Beatmap>> {
    if let Some(b) = md5_from_cache(md5).await {
        return Ok(Some(b));
    }

    if let Some(b) = md5_from_database(db, md5).await? {
        let mut cache = BEATMAP_CACHE.write().await;
        cache.insert(md5.to_string(), b.clone());

        return Ok(Some(b));
    }

    Ok(None)
}

pub async fn md5_from_cache(md5: &str) -> Option<Beatmap> {
    let cache = BEATMAP_CACHE.read().await;

    cache.get(md5).cloned()
}

pub async fn md5_from_database(db: &DbPoolManager, md5: &str) -> Result<Option<Beatmap>> {
    let beatmap = sqlx::query_as::<_, Beatmap>(
        "select server, id, set_id, status, md5, artist, title, version, creator, filename, \
            last_update, total_length, max_combo, frozen, plays, passes, mode, bpm, cs, ar, od, hp, diff \
         from maps where md5 = ?"
    )
        .bind(md5)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(beatmap)
}
