use std::sync::LazyLock;

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    dto::friends::GetFriends, models::User, repository, state::AppState,
    usecases::password::verify_password,
};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

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
        "Hi, it's peppyDonald, you have now found an unused/useless route. Do you want a medal?",
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

    // Serve the map rather than redirect to osu.ppy.sh. That redirect was
    // `Redirect::permanent`, i.e. HTTP 308, which refx-stable cannot follow:
    // osu!win is .NET Framework 4.8, whose HttpWebRequest follows 301/302/303/307
    // but not 308 (that arrived in .NET Core 3.0). The client got a 308 with a
    // zero-length body and failed to import any map it had not already cached.
    //
    // beatmap-service already holds the file, so proxying it is both one hop
    // fewer and independent of whether osu.ppy.sh is reachable from the client.
    let url = format!(
        "{}/v1/get-osu/{}?md5={}",
        state.config.omajinai.beatmap_service_url, beatmap.id, beatmap.md5
    );
    match CLIENT.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.bytes().await {
            Ok(data) => data.into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        },
        _ => StatusCode::NOT_FOUND.into_response(),
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

    match state.storage.load_osz(mapset_id).await {
        Ok(osz) => Bytes::from(osz).into_response(),
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

// redirects -> frontend or other

pub async fn post_difficulty_rating() -> impl IntoResponse {
    Redirect::permanent("https://osu.ppy.sh/difficulty-rating").into_response()
}

pub async fn get_redirect_beatmap(Path(map_id): Path<String>) -> impl IntoResponse {
    Redirect::permanent(&format!("https://refx.online/beatmaps/{map_id}")).into_response()
}

pub async fn get_redirect_profile(Path(user_id): Path<String>) -> impl IntoResponse {
    Redirect::permanent(&format!("https://refx.online/u/{user_id}")).into_response()
}

pub async fn get_redirect_frontend() -> impl IntoResponse {
    Redirect::permanent("https://refx.online").into_response()
}
