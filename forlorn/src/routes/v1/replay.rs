use axum::{
    body::Body,
    extract::{Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::{dto::v1::replay::GetReplay, state::AppState};

pub async fn get_replay(
    State(state): State<AppState>,
    Query(replay): Query<GetReplay>,
) -> impl IntoResponse {
    let replay_data = match state.storage.load_replay(replay.score_id).await {
        Ok(data) => data,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}.osr\"", replay.score_id),
        )
        .body(Body::from(replay_data))
        .unwrap()
        .into_response()
}
