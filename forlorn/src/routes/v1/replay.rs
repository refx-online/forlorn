use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderName, StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::utils::{build_osr_replay, generate_lazer_info};

const CONTENT_DESCRIPTION: HeaderName = HeaderName::from_static("content-description");
use sqlx::Row;

use crate::{dto::v1::replay::GetReplay, state::AppState};

pub async fn get_replay(
    State(state): State<AppState>,
    Query(replay): Query<GetReplay>,
) -> impl IntoResponse {
    let raw_replay = match state.storage.load_replay(replay.score_id).await {
        Ok(data) => data,
        Err(_) => {
            return (StatusCode::NOT_FOUND, "Replay not found.").into_response();
        },
    };

    if !replay.include_headers {
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(CONTENT_DESCRIPTION, "File Transfer")
            .body(Body::from(raw_replay))
            .unwrap()
            .into_response();
    }

    let row = match sqlx::query(
        "SELECT u.name username, m.md5 map_md5, \
         m.artist, m.title, m.version, \
         s.mode, s.n300, s.n100, s.n50, s.ngeki, \
         s.nkatu, s.nmiss, s.score, s.max_combo, \
         s.perfect, s.mods, s.play_time, s.clock_rate \
         FROM scores s \
         INNER JOIN users u ON u.id = s.userid \
         INNER JOIN maps m ON m.md5 = s.map_md5 \
         WHERE s.id = ?",
    )
    .bind(replay.score_id as i64)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Score not found.").into_response();
        },
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        },
    };

    let mode: i32 = row.get("mode");
    let n300: i32 = row.get("n300");
    let n100: i32 = row.get("n100");
    let n50: i32 = row.get("n50");
    let ngeki: i32 = row.get("ngeki");
    let nkatu: i32 = row.get("nkatu");
    let nmiss: i32 = row.get("nmiss");
    let score: i32 = row.get("score");
    let max_combo: i32 = row.get("max_combo");
    let perfect: bool = row.get("perfect");
    let mods: i32 = row.get("mods");
    let play_time: chrono::DateTime<chrono::Utc> = row.get("play_time");
    let map_md5: String = row.get("map_md5");
    let username: String = row.get("username");
    let artist: String = row.get("artist");
    let title: String = row.get("title");
    let version: String = row.get("version");
    let clock_rate: f64 = row.get("clock_rate");

    let lazer_info = if mode >= 12 {
        Some(generate_lazer_info(
            replay.score_id,
            mode,
            mods,
            clock_rate,
            n300,
            n100,
            n50,
            ngeki,
            nkatu,
            nmiss,
        ))
    } else {
        None
    };

    let osr = build_osr_replay(
        &raw_replay,
        mode,
        n300,
        n100,
        n50,
        ngeki,
        nkatu,
        nmiss,
        score,
        max_combo,
        perfect,
        mods,
        play_time,
        replay.score_id,
        &map_md5,
        &username,
        lazer_info.as_deref(),
    );

    // NOTE: these come from the db and end up inside a quoted header value,
    // so strip quotes, backslashes and control chars (response splitting /
    // malformed header via crafted username or map metadata).
    fn sanitize_filename_component(s: &str) -> String {
        s.chars()
            .filter(|c| !c.is_control() && *c != '"' && *c != '\\')
            .collect()
    }

    let filename = format!(
        "{username} - {artist} - {title} [{version}].osr",
        username = sanitize_filename_component(&username),
        artist = sanitize_filename_component(&artist),
        title = sanitize_filename_component(&title),
        version = sanitize_filename_component(&version),
    );

    match Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(CONTENT_DESCRIPTION, "File Transfer")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(osr))
    {
        Ok(res) => res.into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to build replay response",
        )
            .into_response(),
    }
}
