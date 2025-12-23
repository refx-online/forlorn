use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_extra::response::file_stream::FileStream;

use crate::{repository, state::AppState};

const PRIVATE_INITIAL_MAP_ID: i32 = 1000000000;

pub async fn get_peppy() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn get_updated_beatmap(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    let beatmap = match repository::beatmap::fetch_by_filename(&state.db, &filename).await {
        Ok(Some(beatmap)) => beatmap,
        _ => return (StatusCode::OK, b"error: map").into_response(),
    };

    if beatmap.id < PRIVATE_INITIAL_MAP_ID {
        return Redirect::permanent(&format!("https://osu.ppy.sh/web/maps/{filename}"))
            .into_response();
    }

    let file = state.storage.beatmap_file(beatmap.id);

    match FileStream::from_path(file).await {
        Ok(osu) => osu.into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_bancho_connect() -> impl IntoResponse {
    StatusCode::OK
}

// todo: implement this? and maybe move to assets-service
pub async fn get_check_updates() -> impl IntoResponse {
    StatusCode::OK
}
