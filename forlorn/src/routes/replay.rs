use std::time::Instant;

use axum::{
    body::Bytes,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    dto::replay::GetReplay, models::User, repository, state::AppState,
    usecases::password::verify_password,
};

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

pub async fn get_replay(
    State(state): State<AppState>,
    Query(replay): Query<GetReplay>,
) -> impl IntoResponse {
    let now = Instant::now();

    let score = match repository::score::fetch_by_id(&state.db, replay.score_id).await {
        Ok(Some(s)) => s,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };

    let user = match authenticate_user(&state, &replay.password_md5, &replay.username).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    if user.id != score.userid {
        tokio::spawn(async move {
            let _ = repository::stats::increment_replay_views(
                state.db.clone(),
                score.userid,
                score.mode,
            )
            .await;
        });
    }

    let done = now.elapsed();

    let _ = state.metrics.timing(
        "replay.served_elapsed",
        done.as_millis() as i64,
        ["status:all"],
    );

    tracing::info!("Replay served to {} in {}ms.", user.name, done.as_millis());

    match state.storage.load_replay(replay.score_id).await {
        Ok(replay) => Bytes::from(replay).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
