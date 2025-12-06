use std::time::Instant;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    constants::{LeaderboardType, RankedStatus},
    dto::leaderboard::GetScores,
    models::{PersonalBest, User},
    repository,
    state::AppState,
    usecases::password::verify_password,
    utils::{build_display_name, build_empty_leaderboard, build_leaderboard_response},
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

async fn handle_missing_beatmap(state: &AppState, leaderboard: &GetScores) -> Response {
    let has_set_id = leaderboard.map_set_id > 0;

    if !has_set_id {
        state.unsubmitted_maps.insert(leaderboard.map_md5.clone());

        return (StatusCode::OK, b"-1|false").into_response();
    }

    let map_exists = repository::beatmap::fetch_by_filename(&state.db, &leaderboard.map_filename)
        .await
        .ok()
        .flatten()
        .is_some();

    if map_exists {
        state.needs_update_maps.insert(leaderboard.map_md5.clone());

        (StatusCode::OK, b"1|false").into_response()
    } else {
        state.unsubmitted_maps.insert(leaderboard.map_md5.clone());

        (StatusCode::OK, b"-1|false").into_response()
    }
}

pub async fn get_scores(
    State(state): State<AppState>,
    Query(leaderboard): Query<GetScores>,
) -> impl IntoResponse {
    if leaderboard.aqn_files_found() {
        tracing::warn!(
            "aqn files detected for user: {} on map: {}",
            leaderboard.username,
            leaderboard.map_md5
        );

        // restrict? but no one uses aqn tho..
    }

    let now = Instant::now();

    if state.unsubmitted_maps.contains(&leaderboard.map_md5) {
        return (StatusCode::OK, b"-1|false").into_response();
    }
    if state.needs_update_maps.contains(&leaderboard.map_md5) {
        return (StatusCode::OK, b"1|false").into_response();
    }

    let user =
        match authenticate_user(&state, &leaderboard.password_md5, &leaderboard.username).await {
            Ok(user) => user,
            Err(response) => return response,
        };

    let mode = leaderboard.mode();
    //let mods = leaderboard.mods();
    let leaderboard_type = LeaderboardType::from_i32(leaderboard.leaderboard_type);

    let beatmap = match repository::beatmap::fetch_by_md5(&state.db, &leaderboard.map_md5).await {
        Ok(Some(beatmap)) => beatmap,
        Ok(None) => {
            return handle_missing_beatmap(&state, &leaderboard).await;
        },
        Err(_) => return (StatusCode::OK, b"error: db").into_response(),
    };

    if beatmap.status < RankedStatus::Ranked.as_i32() {
        return (
            StatusCode::OK,
            format!("{}|false", beatmap.status).into_bytes(),
        )
            .into_response();
    }

    if leaderboard.requesting_from_editor() {
        return (
            StatusCode::OK,
            build_empty_leaderboard(&beatmap, &state).await,
        )
            .into_response();
    }

    let friend_ids = if leaderboard_type == LeaderboardType::Friends {
        let mut friends = repository::user::fetch_friend_ids(&state.db, user.id)
            .await
            .unwrap_or_default();

        friends.push(user.id);
        Some(friends)
    } else {
        None
    };

    let scores = match repository::leaderboard::fetch_leaderboard_scores(
        &state.db,
        &leaderboard.map_md5,
        mode.as_i32(),
        user.id,
        leaderboard_type,
        Some(leaderboard.mods),
        Some(&user.country),
        friend_ids.as_deref(),
        user.preferred_metric(),
        leaderboard.is_refx(),
    )
    .await
    {
        Ok(s) => s,
        Err(_) => return (StatusCode::OK, b"error: scores").into_response(),
    };

    let personal_best = if !scores.is_empty() {
        match repository::leaderboard::fetch_personal_best_score(
            &state.db,
            &leaderboard.map_md5,
            mode.as_i32(),
            user.id,
            user.preferred_metric(),
            leaderboard.is_refx(),
        )
        .await
        {
            Ok(Some(mut pb)) => {
                let rank = repository::leaderboard::fetch_personal_best_rank(
                    &state.db,
                    &leaderboard.map_md5,
                    mode.as_i32(),
                    pb.preferred_metric,
                    user.preferred_metric(),
                )
                .await
                .unwrap_or(0);

                pb.name = build_display_name(&user, &state).await;
                pb.userid = user.id;

                Some(PersonalBest { score: pb, rank })
            },
            _ => None,
        }
    } else {
        None
    };

    let avg_rating = repository::beatmap::fetch_average_rating(&state.db, &leaderboard.map_md5)
        .await
        .unwrap_or(0.0);

    let leaderboard_response = build_leaderboard_response(
        &beatmap,
        &scores,
        personal_best,
        avg_rating,
        leaderboard.is_refx(),
    );

    let done = now.elapsed();

    tracing::info!(
        "[{}] Leaderboard served to {} in {}ms.",
        mode.as_str(),
        user.name,
        done.as_millis()
    );

    (StatusCode::OK, leaderboard_response.into_bytes()).into_response()
}
