use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{dto::error::GetError, models::ClientError, repository, state::AppState};

pub async fn get_error(
    State(state): State<AppState>,
    Form(error): Form<GetError>,
) -> impl IntoResponse {
    let user_id: i32 = match error.user_id {
        Some(id) => id,
        None => -1,
    };

    let mut client_error = ClientError::from_error(error);

    let username = match repository::user::fetch_by_id(&state.db, &user_id).await {
        Ok(Some(user)) => user.name(),
        _ => "Offline user".into(),
    };

    if user_id != -1 {
        client_error.username = username;
    }

    tracing::info!(
        "{} sent exception: {:?} ({:?})",
        client_error.username,
        client_error.feedback,
        client_error.exception
    );

    tracing::info!(
        "{} sent stacktrace ({:?})",
        client_error.username,
        client_error.stacktrace
    );

    tokio::spawn(async move {
        let _ = repository::error::insert(&state.db, &client_error).await;
    });

    (StatusCode::OK).into_response()
}
