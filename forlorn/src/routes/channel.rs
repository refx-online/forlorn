use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    dto::channel::GetMarkChannelAsRead, models::User, repository, state::AppState,
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

pub async fn mark_as_read(
    State(state): State<AppState>,
    Query(channel): Query<GetMarkChannelAsRead>,
) -> impl IntoResponse {
    let user = match authenticate_user(&state, &channel.password_md5, &channel.username).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    if let Some(target_name) = channel.target {
        let target = match repository::user::fetch_by_name(&state.db, &target_name).await {
            Ok(Some(user)) => user,
            _ => return StatusCode::OK.into_response(),
        };

        tokio::spawn(async move {
            let _ =
                repository::user::mark_conversation_as_read(&state.db, target.id, user.id).await;
        });
    }

    (StatusCode::OK, b"").into_response()
}
