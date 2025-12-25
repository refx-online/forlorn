//use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::response::file_stream::FileStream;

use crate::{
    dto::friends::GetFriends, models::User, repository, state::AppState,
    usecases::password::verify_password,
};

const PRIVATE_INITIAL_MAP_ID: i32 = 1000000000;
const PRIVATE_INITIAL_SET_ID: i32 = 1000000000;

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

pub async fn get_peppy() -> impl IntoResponse {
    (
        StatusCode::OK,
        b"Hi, it's peppyDonald, you have now found an unused/useless route. Do you want a medal?",
    )
        .into_response()
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

pub async fn get_osz(
    State(state): State<AppState>,
    Path(mapset_id): Path<String>,
) -> impl IntoResponse {
    let mapset_id: i32 = match mapset_id.parse() {
        Ok(id) => id,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    if mapset_id < PRIVATE_INITIAL_SET_ID {
        return Redirect::permanent(&format!("{}/d/{mapset_id}", state.config.mirror_endpoint))
            .into_response();
    }

    let file = state.storage.osz_file(mapset_id);

    match FileStream::from_path(file).await {
        Ok(osz) => osz.into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

// todo: implement this? and maybe move to assets-service
pub async fn get_check_updates() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn get_friends(
    State(state): State<AppState>,
    Query(query): Query<GetFriends>,
) -> impl IntoResponse {
    let user = match authenticate_user(&state, &query.password_md5, &query.username).await {
        Ok(user) => user,
        Err(resp) => return resp,
    };

    let friends = match repository::user::fetch_friend_ids(&state.db, user.id).await {
        Ok(ids) => ids,
        Err(_) => return (StatusCode::OK, b"error: db").into_response(),
    };

    let friend = friends
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    (StatusCode::OK, friend).into_response()
}

pub async fn get_root() -> impl IntoResponse {
    /*
    // yes, isui covers are peak
    let banger_songs = [
        "https://www.youtube.com/watch?v=_tYbmNb4VVQ",
        "https://www.youtube.com/watch?v=4QDG2BWxLdE",
        "https://www.youtube.com/watch?v=qlI-HrohYtQ",
        "https://www.youtube.com/watch?v=8GeB7xZqJ48",
        "https://www.youtube.com/watch?v=ML_SkKyZLbA",
        "https://www.youtube.com/watch?v=U09UQ4TXNEQ",
        "https://youtu.be/766qmHTc2ro?si=Pjiq8VIjV22AJtdG",
    ];

    // HACK: use system time as "random"
    let i = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as usize
        % banger_songs.len();

    Redirect::permanent(banger_songs[i]).into_response()
    */

    Redirect::permanent("https://remeliah.cyou").into_response()
}

// redirects -> frontend or other

pub async fn post_difficulty_rating() -> impl IntoResponse {
    Redirect::permanent("https://osu.ppy.sh/difficulty-rating").into_response()
}

pub async fn get_redirect_beatmap(Path(map_id): Path<String>) -> impl IntoResponse {
    Redirect::permanent(&format!("https://remeliah.cyou/beatmaps/{map_id}")).into_response()
}

pub async fn get_redirect_profile(Path(user_id): Path<String>) -> impl IntoResponse {
    Redirect::permanent(&format!("https://remeliah.cyou/u/{user_id}")).into_response()
}
