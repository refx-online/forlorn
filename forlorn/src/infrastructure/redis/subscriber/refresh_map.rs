use anyhow::Result;

use crate::{infrastructure::database::DbPoolManager, repository};

pub async fn refresh_map(db: &DbPoolManager, md5: &str) -> Result<()> {
    {
        let mut cache = repository::beatmap::BEATMAP_CACHE.write().await;
        cache.remove(md5);
    }

    if let Some(bmap) = repository::beatmap::md5_from_database(db, md5).await? {
        let mut cache = repository::beatmap::BEATMAP_CACHE.write().await;
        cache.insert(md5.to_string(), bmap);

        tracing::info!("beatmap {} refreshed!", md5);
    }

    Ok(())
}
