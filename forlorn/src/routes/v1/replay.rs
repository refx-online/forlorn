use axum::{
    body::Bytes,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::state::AppState;

pub async fn get_replay(
    State(state): State<AppState>,
    Query(score_id): Query<u64>,
) -> impl IntoResponse {
    match state.storage.load_replay(score_id).await {
        Ok(replay) => Bytes::from(replay).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
