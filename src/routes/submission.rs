use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

use crate::models::Score;
use crate::models::User;
use crate::repository;
use crate::state::AppState;
use crate::usecases::score::{calculate_accuracy, decrypt_score_data};
use crate::usecases::password::verify_password;

#[derive(Default)]
struct SubmissionFields {
    exited_out: bool,
    fail_time: i32,
    visual_settings_b64: String,
    updated_beatmap_hash: String,
    storyboard_md5: Option<String>,
    iv_b64: Vec<u8>,
    unique_ids: String,
    score_time: i32,
    password_md5: String,
    osu_version: String,
    client_hash_b64: Vec<u8>,

    aim_value: i32,
    ar_value: f32,
    aim: bool,
    arc: bool,
    hdr: bool,
    cs: bool,
    tw: bool,
    twval: f32,
    refx: bool,

    score_data_b64: String,
    username: String,
}

async fn authenticate_user(
    state: &AppState,
    fields: &SubmissionFields,
) -> Result<User, axum::response::Response> {
    let user = match repository::user::fetch_by_name(&state.db, fields.username.trim()).await {
        Ok(Some(user)) => user,
        _ => {
            return Err(StatusCode::OK.into_response());
        },
    };

    match verify_password(&fields.password_md5, &user.pw_bcrypt).await {
        Ok(true) => Ok(user),
        _ => Err(StatusCode::OK.into_response()),
    }
}

pub async fn submit_score(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let _token = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if user_agent != "osu!" {
        // todo: restrict?
        return (StatusCode::BAD_REQUEST, "error: user-agent").into_response();
    }

    let mut fields = SubmissionFields::default();

    // NOTE: unfortunately on axum, i dont have better idea to do like fastapi do
    //       is this really the correct way to parse multipart form data...?
    while let Some(field) = multipart.next_field().await.ok().flatten() {
        let name = field.name().unwrap_or("").to_string();

        let content = field.bytes().await.ok().unwrap_or_default();

        let text = String::from_utf8_lossy(&content);

        match name.as_str() {
            "x" => fields.exited_out = text == "1",
            "ft" => fields.fail_time = text.parse().unwrap_or(0),
            "fs" => fields.visual_settings_b64 = text.to_string(),
            "bmk" => fields.updated_beatmap_hash = text.to_string(),
            "sbk" => fields.storyboard_md5 = Some(text.to_string()),
            "iv" => fields.iv_b64 = content.to_vec(),
            "c1" => fields.unique_ids = text.to_string(),
            "st" => fields.score_time = text.parse().unwrap_or(0),
            "pass" => fields.password_md5 = text.to_string(),
            "osuver" => fields.osu_version = text.to_string(),
            "s" => fields.client_hash_b64 = content.to_vec(),

            "acval" => fields.aim_value = text.parse().unwrap_or(0),
            "arval" => fields.ar_value = text.parse().unwrap_or(0.0),
            "ac" => fields.aim = text == "1" || text.to_lowercase() == "true",
            "ar" => fields.arc = text == "1" || text.to_lowercase() == "true",
            "hdrem" => fields.hdr = text == "1" || text.to_lowercase() == "true",
            "cs" => fields.cs = text == "1" || text.to_lowercase() == "true",
            "tw" => fields.tw = text == "1" || text.to_lowercase() == "true",
            "twval" => fields.twval = text.parse().unwrap_or(0.0),
            "refx" => fields.refx = text == "1" || text.to_lowercase() == "true",

            "score" => fields.score_data_b64 = text.to_string(),
            "u" => fields.username = text.to_string(),
            _ => {},
        }
    }

    let user = match authenticate_user(&state, &fields).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let (score_data, _) = match decrypt_score_data(
        fields.score_data_b64.as_bytes(),
        &fields.client_hash_b64,
        &fields.iv_b64,
        &fields.osu_version,
    ) {
        Ok((v, c)) => (v, c),
        Err(_) => return (StatusCode::BAD_REQUEST, "error: decrypt").into_response(),
    };

    let beatmap_md5 = score_data.get(0).cloned().unwrap_or_default();

    if beatmap_md5.is_empty() {
        return (StatusCode::BAD_REQUEST, "error: beatmap").into_response();
    }

    let beatmap = match repository::beatmap::fetch_by_md5(&state.db, &beatmap_md5).await {
        Ok(Some(beatmap)) => beatmap,
        _ => return (StatusCode::BAD_REQUEST, "error: beatmap").into_response(),
    };

    let mut score = match Score::from_submission(&score_data[2..]) {
        Some(score) => score,
        None => return (StatusCode::BAD_REQUEST, "error: score").into_response(),
    };

    score.acc = calculate_accuracy(&score);

    let _ = repository::user::update_latest_activity(&state.db, user.id).await;

    // todo

    (StatusCode::OK, "ok").into_response()
}
