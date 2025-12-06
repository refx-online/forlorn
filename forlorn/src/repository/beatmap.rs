use std::{collections::HashMap, sync::LazyLock};

use anyhow::Result;
use tokio::sync::RwLock;

use crate::{
    infrastructure::{
        database::DbPoolManager,
        omajinai::beatmap::{api_get_beatmaps, parse_beatmap_from_api},
    },
    models::Beatmap,
};

pub static BEATMAP_CACHE: LazyLock<RwLock<HashMap<String, Beatmap>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub async fn fetch_by_md5(api_key: &str, db: &DbPoolManager, md5: &str) -> Result<Option<Beatmap>> {
    if let Some(b) = md5_from_cache(md5).await {
        return Ok(Some(b));
    }

    if let Some(b) = md5_from_database(db, md5).await? {
        let mut cache = BEATMAP_CACHE.write().await;
        cache.insert(md5.to_string(), b.clone());

        return Ok(Some(b));
    }

    if let Some(b) = md5_from_api(api_key, db, md5).await? {
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

async fn md5_from_api(api_key: &str, db: &DbPoolManager, md5: &str) -> Result<Option<Beatmap>> {
    let resp = api_get_beatmaps(api_key, Some(md5)).await?;

    if resp.is_none() {
        return Ok(None);
    }

    let beatmaps: Vec<Beatmap> = resp
        .unwrap()
        .into_iter()
        .map(parse_beatmap_from_api)
        .collect();

    for beatmap in &beatmaps {
        tokio::spawn({
            let db = db.clone();
            let b = beatmap.clone();

            async move {
                let _ = save(&db.clone(), &b).await;
            }
        });
    }

    Ok(beatmaps.into_iter().find(|b| b.md5 == md5))
}

pub async fn fetch_by_filename(db: &DbPoolManager, filename: &str) -> Result<Option<Beatmap>> {
    let beatmap = sqlx::query_as::<_, Beatmap>("select * from maps where filename = ?")
        .bind(filename)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(beatmap)
}

pub async fn fetch_average_rating(db: &DbPoolManager, map_md5: &str) -> Result<f32> {
    let avg: Option<f32> = sqlx::query_scalar("select avg(rating) from ratings where map_md5 = ?")
        .bind(map_md5)
        .fetch_optional(db.as_ref())
        .await?
        .flatten();

    Ok(avg.unwrap_or(0.0))
}

async fn save(db: &DbPoolManager, beatmap: &Beatmap) -> Result<()> {
    sqlx::query(
        "replace into maps (server, id, set_id, status, md5, artist, title, version, creator, \
         filename, last_update, total_length, max_combo, frozen, plays, passes, mode, bpm, cs, ar, od, hp, diff) \
         values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&beatmap.server)
    .bind(beatmap.id)
    .bind(beatmap.set_id)
    .bind(beatmap.status)
    .bind(&beatmap.md5)
    .bind(&beatmap.artist)
    .bind(&beatmap.title)
    .bind(&beatmap.version)
    .bind(&beatmap.creator)
    .bind(&beatmap.filename)
    .bind(beatmap.last_update)
    .bind(beatmap.total_length)
    .bind(beatmap.max_combo)
    .bind(beatmap.frozen)
    .bind(beatmap.plays)
    .bind(beatmap.passes)
    .bind(beatmap.mode)
    .bind(beatmap.bpm)
    .bind(beatmap.cs)
    .bind(beatmap.ar)
    .bind(beatmap.od)
    .bind(beatmap.hp)
    .bind(beatmap.diff)
    .execute(db.as_ref())
    .await?;

    Ok(())
}
