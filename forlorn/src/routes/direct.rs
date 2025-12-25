use std::{collections::HashMap, sync::LazyLock};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;

use crate::{
    constants::RankedStatus,
    dto::direct::{GetDirectSearch, GetDirectSearchSet},
    models::{Beatmap, BeatmapChild, BeatmapSet, User},
    repository,
    state::AppState,
    usecases::password::verify_password,
};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

async fn authenticate_user(
    state: &AppState,
    password_md5: &str,
    username: &str,
) -> Result<User, Response> {
    let user = match repository::user::fetch_by_name(&state.db, username).await {
        Ok(Some(user)) => user,
        _ => {
            return Err(StatusCode::OK.into_response());
        },
    };

    match verify_password(password_md5, &user.pw_bcrypt).await {
        Ok(true) => Ok(user),
        _ => Err(StatusCode::OK.into_response()),
    }
}

fn format_direct_map_info(bmap: &Beatmap) -> String {
    format!(
        "[{:.2}⭐] {} {{cs: {} / od: {} / ar: {} / hp: {}}}@{}",
        bmap.diff, bmap.version, bmap.cs, bmap.od, bmap.ar, bmap.hp, bmap.mode
    )
}

fn format_direct_set_info(bmap: &Beatmap, diffs: &str) -> String {
    format!(
        "{}.osz|{}|{}|{}|{}|10.0|{}|{}|0|0|0|0|0|{}",
        bmap.set_id,
        bmap.artist,
        bmap.title,
        bmap.creator,
        bmap.status,
        bmap.last_update.format("%Y-%m-%d %H:%M:%S"),
        bmap.set_id,
        diffs
    )
}

fn beatmap_from_mirror(bmapset: &BeatmapSet, child: &BeatmapChild) -> Beatmap {
    Beatmap {
        id: child.beatmap_id,
        set_id: bmapset.set_id,
        status: bmapset.ranked_status,
        md5: child.file_md5.clone(),
        artist: bmapset.artist.clone(),
        title: bmapset.title.clone(),
        version: child.diff_name.clone(),
        creator: bmapset.creator.clone(),
        filename: String::new(),
        last_update: bmapset.last_update.parse().unwrap_or_else(|_| Utc::now()),
        total_length: child.total_length,
        max_combo: child.max_combo,
        frozen: false,
        plays: child.playcount,
        passes: child.passcount,
        mode: child.mode,
        bpm: child.bpm,
        cs: child.cs,
        ar: child.ar,
        od: child.od,
        hp: child.hp,
        diff: child.difficulty_rating,
    }
}

pub async fn get_direct_search(
    State(state): State<AppState>,
    Query(direct): Query<GetDirectSearch>,
) -> impl IntoResponse {
    let user = match authenticate_user(&state, &direct.password_md5, &direct.username).await {
        Ok(user) => user,
        Err(resp) => return resp,
    };

    let offset = direct.page * 100;
    let mut request_direct = vec![("amount", "100".to_string()), ("offset", offset.to_string())];

    if direct.query != "Newest" && direct.query != "Top+Rated" && direct.query != "Most+Played" {
        request_direct.push(("query", direct.query.clone()));
    }

    if direct.mode != -1 {
        request_direct.push(("mode", direct.mode.to_string()));
    }

    if direct.status != 4 {
        let osu_api_status = RankedStatus::from_osudirect(direct.status).as_osu_api();

        request_direct.push(("status", osu_api_status.to_string()));
    }

    let response = match CLIENT
        .get(format!("{}/api/search", state.config.mirror_endpoint))
        .query(&request_direct)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(_) => {
            return (
                StatusCode::OK,
                b"-1\nFailed to retrieve data from the beatmap mirror.",
            )
                .into_response();
        },
    };

    if response.status() != StatusCode::OK {
        return (
            StatusCode::OK,
            b"-1\nFailed to retrieve data from the beatmap mirror.",
        )
            .into_response();
    }

    let result: Vec<BeatmapSet> = match response.json().await {
        Ok(data) => data,
        Err(_) => {
            return (StatusCode::OK, b"-1\nFailed to parse beatmap data.").into_response();
        },
    };

    let mode = if direct.mode == -1 { None } else { Some(direct.mode) };
    let ranked_status = Some(RankedStatus::from_osudirect(direct.status) as i32);

    let private_bmapsets =
        repository::beatmap::fetch_many(&state.db, mode, ranked_status, direct.page + 1, 100)
            .await
            .unwrap_or_default();

    let mut normalized_result = Vec::new();
    for bmapset in result {
        for child in &bmapset.children_beatmaps {
            normalized_result.push(beatmap_from_mirror(&bmapset, child));
        }
    }

    let mut combined_results = Vec::new();
    combined_results.extend(private_bmapsets);
    combined_results.extend(normalized_result);

    let mut unique_map: HashMap<i32, Beatmap> = HashMap::new();
    for bmap in combined_results {
        unique_map.insert(bmap.set_id, bmap);
    }

    let unique_results: Vec<Beatmap> = unique_map.into_values().collect();

    let count = if unique_results.len() == 100 {
        "101".to_string()
    } else {
        unique_results.len().to_string()
    };

    let mut ret = vec![count];

    for bmap in unique_results {
        let diffs = format_direct_map_info(&bmap);
        ret.push(format_direct_set_info(&bmap, &diffs));
    }

    let _ = state.metrics.incr("direct_served", ["status:ok"]);

    tracing::info!("Served direct search for {} ({})", user.name(), ret.len());

    (StatusCode::OK, ret.join("\n").into_bytes()).into_response()
}

pub async fn get_direct_search_set(
    State(state): State<AppState>,
    Query(direct): Query<GetDirectSearchSet>,
) -> impl IntoResponse {
    let user = match authenticate_user(&state, &direct.password_md5, &direct.username).await {
        Ok(user) => user,
        Err(resp) => return resp,
    };

    let bmapset = if let Some(set_id) = direct.map_set_id {
        repository::beatmap::fetch_set_by_set_id(&state.db, set_id).await
    } else if let Some(map_id) = direct.map_id {
        repository::beatmap::fetch_set_by_map_id(&state.db, map_id).await
    } else if let Some(ref checksum) = direct.map_md5 {
        repository::beatmap::fetch_set_by_md5(&state.db, checksum).await
    } else {
        return (StatusCode::OK, b"").into_response();
    };

    let bmapset = match bmapset {
        Ok(Some(set)) => set,
        _ => {
            // TODO: get from osu!
            return (StatusCode::OK, Vec::new()).into_response();
        },
    };

    let rating = if let Some(ref checksum) = direct.map_md5 {
        repository::rating::fetch_average_rating(&state.db, checksum)
            .await
            .unwrap_or(0.0)
    } else {
        10.0
    };

    let response = format!(
        "{}.osz|{}|{}|{}|{}|{:.1}|{}|{}|0|0|0|0|0",
        bmapset.set_id,
        bmapset.artist,
        bmapset.title,
        bmapset.creator,
        bmapset.status,
        rating,
        bmapset.last_update.format("%Y-%m-%d %H:%M:%S"),
        bmapset.set_id
    );

    let _ = state.metrics.incr("direct_set_served", ["status:ok"]);

    tracing::info!("Served direct search set for {}", user.name());

    (StatusCode::OK, response.into_bytes()).into_response()
}
