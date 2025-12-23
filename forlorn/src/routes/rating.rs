use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    constants::RankedStatus, dto::rating::GetRating, models::User, repository, state::AppState,
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

pub async fn get_rating(
    State(state): State<AppState>,
    Query(rating): Query<GetRating>,
) -> impl IntoResponse {
    let user = match authenticate_user(&state, &rating.password_md5, &rating.username).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    if let Some(rate) = rating.rating {
        // client is submitting a rating for the map
        let _ = repository::rating::insert(&state.db, &rating.map_md5, user.id, rate).await;
    } else {
        let beatmap = match repository::beatmap::fetch_by_md5(
            &state.config,
            &state.db,
            &rating.map_md5,
        )
        .await
        {
            Ok(Some(beatmap)) => beatmap,
            _ => return (StatusCode::OK, b"error: beatmap").into_response(),
        };

        // only allow ranked maps
        if beatmap.status < RankedStatus::Ranked.as_i32() {
            return (StatusCode::OK, b"error: unranked").into_response();
        }

        // osu! client is checking whether we can rate the map or not.
        // the client hasn't rated the map, so simply
        // tell them that they can submit a rating.
        if let Ok(None) = repository::rating::fetch(&state.db, &rating.map_md5, user.id).await {
            return (StatusCode::OK, b"ok").into_response();
        }
    }

    let map_ratings = repository::rating::fetch_many(&state.db, &rating.map_md5).await;
    let avg_rating = map_ratings
        .ok()
        .filter(|ratings| !ratings.is_empty())
        .map(|ratings| {
            let sum: f32 = ratings.iter().map(|(_, r)| f32::from(*r)).sum();

            sum / ratings.len() as f32
        })
        .unwrap_or(0.0);

    (
        StatusCode::OK,
        format!("alreadyvoted\n{}", avg_rating).into_bytes(),
    )
        .into_response()
}
