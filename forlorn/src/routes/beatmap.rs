use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    constants::RankedStatus, dto::beatmap::GetBeatmapInfo, models::User, repository,
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

pub async fn get_beatmap_info(
    State(state): State<AppState>,
    Form(beatmap): Form<GetBeatmapInfo>,
) -> impl IntoResponse {
    let user = match authenticate_user(&state, &beatmap.password_md5, &beatmap.username).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let mut ret: Vec<String> = Vec::new();
    for (idx, filename) in beatmap.filenames.iter().enumerate() {
        let Some(map) = repository::beatmap::fetch_by_filename(&state.db, filename)
            .await
            .ok()
            .flatten()
        else {
            continue;
        };

        let scores = repository::score::fetch_best(
            &state.db,
            user.id,
            map.md5.as_str(),
            // NOTE: izin brok, kita cuman ambil std doang soalnya
            //       gabisa ngambil mode realtime dari cho
            0,
        )
        .await
        .unwrap_or_default();

        let grade = scores
            .into_iter()
            .next()
            .map(|s| s.grade)
            .unwrap_or_else(|| "N".to_string());

        let grades = format!("{}|N|N|N", grade);

        ret.push(format!(
            "{i}|{id}|{set_id}|{md5}|{status}|{grades}",
            i = idx,
            id = map.id,
            set_id = map.set_id,
            md5 = map.md5,
            status = match map.status {
                -2..=0 => RankedStatus::Pending.as_i32(),
                1 => RankedStatus::Ranked.as_i32(),
                2 => RankedStatus::Approved.as_i32(),
                3 => RankedStatus::Qualified.as_i32(),
                4 => RankedStatus::Loved.as_i32(),
                _ => RankedStatus::UpdateAvailable.as_i32(),
            },
            grades = grades,
        ));
    }

    if !beatmap.ids.is_empty() {
        tracing::warn!(
            "{} requested map(s) info by id ({:?})",
            user.name,
            beatmap.ids
        );
    }

    (StatusCode::OK, ret.join("\n")).into_response()
}
