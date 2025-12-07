use std::{collections::HashMap, sync::LazyLock};

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
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
        "select id, set_id, status, md5, artist, title, version, creator, filename, \
            last_update, total_length, max_combo, frozen, plays, passes, mode, bpm, cs, ar, od, hp, diff \
         from maps where md5 = ?"
    )
    .bind(md5)
    .fetch_optional(db.as_ref())
    .await?;

    let beatmap = match beatmap {
        Some(b) => b,
        None => return Ok(None),
    };

    let set: Vec<Beatmap> = sqlx::query_as::<_, Beatmap>(
        "select id, set_id, status, md5, artist, title, version, creator, filename, \
            last_update, total_length, max_combo, frozen, plays, passes, mode, bpm, cs, ar, od, hp, diff \
         from maps where set_id = ?"
    )
    .bind(beatmap.set_id)
    .fetch_all(db.as_ref())
    .await?;

    let last: Option<DateTime<Utc>> = sqlx::query_scalar(
        "select last_osuapi_check from mapsets where id = ? and server = 'osu!'",
    )
    .bind(beatmap.set_id)
    .fetch_optional(db.as_ref())
    .await?;

    if should_update_mapset(&set, last).await {
        return Ok(None); // we force refetch from api
    }

    Ok(Some(beatmap))
}

async fn md5_from_api(api_key: &str, db: &DbPoolManager, md5: &str) -> Result<Option<Beatmap>> {
    let resp = match api_get_beatmaps(api_key, Some(md5), None).await? {
        Some(r) if !r.is_empty() => r,
        _ => return Ok(None),
    };

    let set_id: i32 = resp[0].set_id.parse().unwrap_or(0);

    let set_resp = match api_get_beatmaps(api_key, None, Some(&set_id)).await? {
        Some(r) if !r.is_empty() => r,
        _ => return Ok(None),
    };

    let api_beatmaps: Vec<Beatmap> = set_resp
        .into_iter()
        .map(|d| parse_beatmap_from_api(d, api_key.is_empty())) // stupid
        .collect();

    let existing_maps: HashMap<i32, Beatmap> =
        sqlx::query_as::<_, Beatmap>("select * from maps where set_id = ?")
            .bind(set_id)
            .fetch_all(db.as_ref())
            .await?
            .into_iter()
            .map(|b| (b.id, b))
            .collect();

    // TODO: remove stale maps

    for beatmap in &api_beatmaps {
        let mut m = beatmap.clone();
        if let Some(existing) = existing_maps.get(&beatmap.id) {
            m.plays = existing.plays;
            m.passes = existing.passes;
            m.frozen = existing.frozen;

            // this sounds right
            if !existing.has_leaderboard() {
                m.status = beatmap.status;
            } else {
                m.status = existing.status;
            }
        }

        save(db, &m).await?;
    }

    sqlx::query("replace into mapsets (id, server, last_osuapi_check) values (?, ?, ?)")
        .bind(set_id)
        .bind("osu!")
        .bind(Utc::now())
        .execute(db.as_ref())
        .await?;

    Ok(api_beatmaps.into_iter().find(|b| b.md5 == md5))
}

pub async fn fetch_by_filename(db: &DbPoolManager, filename: &str) -> Result<Option<Beatmap>> {
    let beatmap = sqlx::query_as::<_, Beatmap>(
        "select id, set_id, status, md5, artist, title, version, creator, filename, \
            last_update, total_length, max_combo, frozen, plays, passes, mode, bpm, cs, ar, od, hp, diff \
         from maps where filename = ?"
    )
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

async fn should_update_mapset(
    beatmaps: &[Beatmap],
    last_osuapi_check: Option<DateTime<Utc>>,
) -> bool {
    if beatmaps.is_empty() {
        return true;
    }

    let last_map_update = beatmaps.iter().map(|b| b.last_update).max().unwrap();

    let update = Utc::now() - last_map_update;

    let mut check = Duration::hours(2) + Duration::hours(5 * update.num_days() / 365);

    let max = Duration::days(1);
    if check > max {
        check = max;
    }

    match last_osuapi_check {
        Some(last) => Utc::now() > last + check,
        None => true,
    }
}

async fn save(db: &DbPoolManager, beatmap: &Beatmap) -> Result<()> {
    sqlx::query(
        "replace into maps (server, id, set_id, status, md5, artist, title, version, creator, \
         filename, last_update, total_length, max_combo, frozen, plays, passes, mode, bpm, cs, ar, od, hp, diff) \
         values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("osu!") // TODO: private?
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
