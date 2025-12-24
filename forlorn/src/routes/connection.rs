use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use crate::{
    dto::connection::GetBanchoConnect, geoloc::fetch_geoloc, models::User, repository,
    state::AppState, usecases::password::verify_password,
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

pub async fn get_bancho_connect(
    State(state): State<AppState>,
    Query(connect): Query<GetBanchoConnect>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user = match authenticate_user(&state, &connect.password_md5, &connect.username).await {
        Ok(user) => user,
        Err(resp) => return resp,
    };

    if user.country != "xx" {
        return (StatusCode::OK, b"").into_response();
    }

    // we won't handle the actual "connection" here
    // since we depends on cho
    // but we can at least fix the country code for users with "xx" country.

    if let Some(geoloc) = fetch_geoloc(&headers).await {
        match repository::user::update_country(&state.db, user.id, &geoloc.country_code).await {
            Ok(_) => {
                tracing::info!(
                    "Updated country for {} to {} (lat: {}, lon: {})",
                    user.name(),
                    geoloc.country_code,
                    geoloc.latitude,
                    geoloc.longitude
                );
            },
            Err(_) => return (StatusCode::OK, b"error: db").into_response(),
        };
    }

    StatusCode::OK.into_response()
}
