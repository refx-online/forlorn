use axum::{
    body::Bytes,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{dto::v1::replay::GetReplay, state::AppState};

pub async fn get_replay(
    State(state): State<AppState>,
    Query(replay): Query<GetReplay>,
) -> impl IntoResponse {
    match state.storage.load_replay(replay.score_id).await {
        Ok(replay) => Bytes::from(replay).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
