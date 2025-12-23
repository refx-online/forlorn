use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    dto::favourite::{AddFavourites, GetFavourites},
    models::User,
    repository,
    state::AppState,
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

pub async fn get_favourites(
    State(state): State<AppState>,
    Query(favourite): Query<GetFavourites>,
) -> impl IntoResponse {
    let user = match authenticate_user(&state, &favourite.password_md5, &favourite.username).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let favourites = match repository::favourite::fetch_all(&state.db, user.id).await {
        Ok(favs) => favs,
        Err(_) => return (StatusCode::OK, b"error: db").into_response(),
    };

    let favourite = favourites
        .into_iter()
        .map(|f| f.setid.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    (StatusCode::OK, favourite).into_response()
}

pub async fn add_favourites(
    State(state): State<AppState>,
    Query(favourite): Query<AddFavourites>,
) -> impl IntoResponse {
    let user = match authenticate_user(&state, &favourite.password_md5, &favourite.username).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    if let Ok(existing) =
        repository::favourite::fetch_one(&state.db, user.id, favourite.mapset_id).await
        && existing.is_some()
    {
        return (StatusCode::OK, b"error: already favourited").into_response();
    }

    if repository::favourite::insert(&state.db, user.id, favourite.mapset_id)
        .await
        .is_err()
    {
        return (StatusCode::OK, b"error: db").into_response();
    }

    (StatusCode::OK, b"Added favourite!").into_response()
}
