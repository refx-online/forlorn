use std::{collections::HashMap, sync::LazyLock};

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use tokio::sync::RwLock;

use crate::{
    config::Config,
    infrastructure::{
        database::DbPoolManager,
        omajinai::beatmap::{api_get_beatmaps, parse_beatmap_from_api, update_beatmap_from_api},
    },
    models::{Beatmap, BeatmapSetInfo},
};

// this cache has:
// map_id as key
// map_md5 as key
// TODO: separate this?
pub static BEATMAP_CACHE: LazyLock<RwLock<HashMap<String, Beatmap>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

const PRIVATE_INITIAL_SET_ID: i32 = 1000000000;

pub async fn fetch_by_md5(
    config: &Config,
    db: &DbPoolManager,
    md5: &str,
) -> Result<Option<Beatmap>> {
    if let Some(b) = md5_from_cache(md5).await {
        return Ok(Some(b));
    }

    if let Some(b) = md5_from_database(db, md5).await? {
        let mut cache = BEATMAP_CACHE.write().await;
        cache.insert(md5.to_string(), b.clone());

        return Ok(Some(b));
    }

    if let Some(b) = md5_from_api(config, db, md5).await? {
        let mut cache = BEATMAP_CACHE.write().await;
        cache.insert(md5.to_string(), b.clone());

        return Ok(Some(b));
    }

    Ok(None)
}

pub async fn fetch_by_id(db: &DbPoolManager, id: &i32) -> Result<Option<Beatmap>> {
    if let Some(b) = id_from_cache(id).await {
        return Ok(Some(b));
    }

    if let Some(b) = id_from_database(db, id).await? {
        let mut cache = BEATMAP_CACHE.write().await;
        cache.insert(id.to_string(), b.clone());

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

    // we wont let our "private" maps to touch osu api for obvious reason
    // todo: use server?
    if beatmap.set_id >= PRIVATE_INITIAL_SET_ID {
        return Ok(Some(beatmap));
    }

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
        let mut cache = BEATMAP_CACHE.write().await;
        cache.retain(|_, b| b.set_id != beatmap.set_id);

        return Ok(None); // we force refetch from api
    }

    let mut cache = BEATMAP_CACHE.write().await;
    for b in set {
        // might as well cache them all
        cache.insert(b.md5.clone(), b);
    }

    drop(cache);

    Ok(Some(beatmap))
}

async fn md5_from_api(config: &Config, db: &DbPoolManager, md5: &str) -> Result<Option<Beatmap>> {
    let resp = match api_get_beatmaps(config, Some(md5), None).await? {
        Some(r) if !r.is_empty() => r,
        _ => {
            // API returned 404, map deleted.
            // we can safe to assume that the map is deleted, we should delete them in db.
            let set_id_opt: Option<i32> =
                sqlx::query_scalar("select set_id from maps where md5 = ?")
                    .bind(md5)
                    .fetch_optional(db.as_ref())
                    .await?;

            if let Some(set_id) = set_id_opt {
                sqlx::query("delete from scores where map_md5 = ?")
                    .bind(md5)
                    .execute(db.as_ref())
                    .await?;

                sqlx::query("delete from maps where md5 = ?")
                    .bind(md5)
                    .execute(db.as_ref())
                    .await?;

                let remaining: i64 =
                    sqlx::query_scalar("select count(*) from maps where set_id = ?")
                        .bind(set_id)
                        .fetch_one(db.as_ref())
                        .await?;

                if remaining == 0 {
                    sqlx::query("delete from mapsets where id = ?")
                        .bind(set_id)
                        .execute(db.as_ref())
                        .await?;
                }
                let mut cache = BEATMAP_CACHE.write().await;
                cache.remove(md5);
            }

            return Ok(None);
        },
    };

    let set_id: i32 = resp[0].set_id.parse().unwrap_or(0);

    let set_resp = match api_get_beatmaps(config, None, Some(&set_id)).await? {
        Some(r) if !r.is_empty() => r,
        _ => return Ok(None),
    };

    let existing_maps: HashMap<i32, Beatmap> =
        sqlx::query_as::<_, Beatmap>("select * from maps where set_id = ?")
            .bind(set_id)
            .fetch_all(db.as_ref())
            .await?
            .into_iter()
            .map(|b| (b.id, b))
            .collect();

    let api_map_ids: Vec<i32> = set_resp.iter().map(|d| d.id.parse().unwrap_or(0)).collect();

    let stale_maps: Vec<&Beatmap> = existing_maps
        .values()
        .filter(|b| !api_map_ids.contains(&b.id))
        .collect();

    if !stale_maps.is_empty() {
        for bmap in &stale_maps {
            sqlx::query("delete from scores where map_md5 = ?")
                .bind(&bmap.md5)
                .execute(db.as_ref())
                .await?;

            sqlx::query("delete from maps where md5 = ?")
                .bind(&bmap.md5)
                .execute(db.as_ref())
                .await?;
        }
    }

    let mut to_save = Vec::new();
    for beatmap in &set_resp {
        let map_id: i32 = beatmap.id.parse().unwrap_or(0);

        if let Some(existing) = existing_maps.get(&map_id) {
            let mut updated = existing.clone();
            update_beatmap_from_api(&mut updated, beatmap, config.osu.api_key.is_empty());

            to_save.push(updated);
        } else {
            let mut new_map =
                parse_beatmap_from_api(beatmap.clone(), config.osu.api_key.is_empty());

            new_map.frozen = false;
            new_map.plays = 0;
            new_map.passes = 0;

            to_save.push(new_map);
        }
    }

    save(db, &to_save).await?;

    sqlx::query("replace into mapsets (id, server, last_osuapi_check) values (?, ?, ?)")
        .bind(set_id)
        .bind("osu!")
        .bind(Utc::now())
        .execute(db.as_ref())
        .await?;

    Ok(to_save.into_iter().find(|b| b.md5 == md5))
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

pub async fn fetch_many(
    db: &DbPoolManager,
    mode: Option<i32>,
    ranked_status: Option<i32>,
    page: i32,
    page_size: i64,
) -> Result<Vec<Beatmap>> {
    let mut query = String::from(
        "select id, set_id, status, md5, artist, title, version, creator, filename, \
         last_update, total_length, max_combo, frozen, plays, passes, mode, bpm, cs, ar, od, hp, diff \
         from maps",
    );

    let mut cond = Vec::new();

    if let Some(m) = mode {
        cond.push(format!("mode = {}", m));
    }

    if let Some(rs) = ranked_status {
        cond.push(format!("status = {}", rs));
    }

    if !cond.is_empty() {
        query.push_str(" where ");
        query.push_str(&cond.join(" and "));
    }

    let page = page.max(1);
    let limit = page_size.clamp(1, 1000) as i32;
    let offset = (page - 1) * limit;

    query.push_str(&format!(" limit {} offset {}", limit, offset));

    let beatmaps = sqlx::query_as::<_, Beatmap>(&query)
        .fetch_all(db.as_ref())
        .await?;

    Ok(beatmaps)
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

pub async fn fetch_set_by_set_id(
    db: &DbPoolManager,
    set_id: i32,
) -> Result<Option<BeatmapSetInfo>> {
    let result = sqlx::query_as::<_, BeatmapSetInfo>(
        "select distinct set_id, artist, title, status, creator, last_update \
         from maps where set_id = ?",
    )
    .bind(set_id)
    .fetch_optional(db.as_ref())
    .await?;

    Ok(result)
}

pub async fn fetch_set_by_map_id(
    db: &DbPoolManager,
    map_id: i32,
) -> Result<Option<BeatmapSetInfo>> {
    let result = sqlx::query_as::<_, BeatmapSetInfo>(
        "select distinct set_id, artist, title, status, creator, last_update \
         from maps where id = ?",
    )
    .bind(map_id)
    .fetch_optional(db.as_ref())
    .await?;

    Ok(result)
}

pub async fn fetch_set_by_md5(db: &DbPoolManager, md5: &String) -> Result<Option<BeatmapSetInfo>> {
    let result = sqlx::query_as::<_, BeatmapSetInfo>(
        "select distinct set_id, artist, title, status, creator, last_update \
         from maps where md5 = ?",
    )
    .bind(md5)
    .fetch_optional(db.as_ref())
    .await?;

    Ok(result)
}

pub async fn id_from_cache(id: &i32) -> Option<Beatmap> {
    let cache = BEATMAP_CACHE.read().await;

    cache.get(&id.to_string()).cloned()
}

pub async fn id_from_database(db: &DbPoolManager, map_id: &i32) -> Result<Option<Beatmap>> {
    let beatmap = sqlx::query_as::<_, Beatmap>(
        "select id, set_id, status, md5, artist, title, version, creator, filename, \
            last_update, total_length, max_combo, frozen, plays, passes, mode, bpm, cs, ar, od, hp, diff \
         from maps where id = ?"
    )
    .bind(map_id)
    .fetch_optional(db.as_ref())
    .await?;

    let beatmap = match beatmap {
        Some(b) => b,
        None => return Ok(None),
    };

    Ok(Some(beatmap))
}

async fn save(db: &DbPoolManager, beatmaps: &[Beatmap]) -> Result<()> {
    if beatmaps.is_empty() {
        return Ok(());
    }

    let mut tx = db.begin().await?;

    for beatmap in beatmaps {
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
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}
