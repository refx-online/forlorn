use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

use crate::constants::SubmissionStatus;
use crate::models::Score;
use crate::models::User;
use crate::repository;
use crate::state::AppState;
use crate::usecases::beatmap::ensure_local_osu_file;
use crate::usecases::password::verify_password;
use crate::usecases::score::{
    calculate_accuracy, calculate_score_performance, calculate_status, decrypt_score_data,
};

#[derive(Debug, Clone)]
struct ScoreHeader {
    map_md5: String,
    username: String,
}

impl ScoreHeader {
    fn from_decrypted(score_data: &[String]) -> Option<Self> {
        if score_data.len() < 2 {
            return None;
        }

        Some(Self {
            map_md5: score_data[0].clone(),
            username: score_data[1].trim().to_string(),
        })
    }
}

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

    // refx original sin
    aim_value: i32,
    ar_value: f32,
    aim: bool,
    arc: bool,
    hdr: bool,
    cs: bool,
    tw: bool,
    twval: f32,
    refx: bool,
    score_data_b64: Vec<u8>,
    replay_file: Vec<u8>,
}

async fn authenticate_user(
    state: &AppState,
    fields: &SubmissionFields,
    username: &str,
) -> Result<User, axum::response::Response> {
    let user = match repository::user::fetch_by_name(&state.db, username).await {
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
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if user_agent != "osu!" {
        // todo: restrict?
        return (StatusCode::BAD_REQUEST, "error: user-agent").into_response();
    }

    let mut fields = SubmissionFields::default();

    // HACK: to count how many "score" fields are there
    //       because osu! client sends both score data and replay file
    //       with the same field name "score" (pepy why)
    let mut score_count = 0;

    // NOTE: unfortunately on axum, i dont have better idea to do like fastapi do
    //       is this really the correct way to parse multipart form data...?
    while let Some(field) = multipart.next_field().await.ok().flatten() {
        let name = field.name().map(|s| s.to_owned()).unwrap_or_default();
        let content = field.bytes().await.unwrap_or_default();
        let text = String::from_utf8_lossy(&content);

        if name == "score" {
            if score_count == 0 {
                fields.score_data_b64 = content.to_vec();
            } else if score_count == 1 {
                fields.replay_file = content.to_vec();
            }

            score_count += 1;
            continue;
        }

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

            // refx original sin
            "acval" => fields.aim_value = text.parse().unwrap_or(0),
            "arval" => fields.ar_value = text.parse().unwrap_or(0.0),
            "ac" => fields.aim = text == "1" || text.to_lowercase() == "true",
            "ar" => fields.arc = text == "1" || text.to_lowercase() == "true",
            "hdrem" => fields.hdr = text == "1" || text.to_lowercase() == "true",
            "cs" => fields.cs = text == "1" || text.to_lowercase() == "true",
            "tw" => fields.tw = text == "1" || text.to_lowercase() == "true",
            "twval" => fields.twval = text.parse().unwrap_or(0.0),
            "refx" => fields.refx = text == "1" || text.to_lowercase() == "true",
            _ => {},
        }
    }

    let (score_data, _) = match decrypt_score_data(
        &fields.score_data_b64,
        &fields.client_hash_b64,
        &fields.iv_b64,
        &fields.osu_version,
    ) {
        Ok((v, c)) => (v, c),
        Err(_) => return (StatusCode::BAD_REQUEST, "error: decrypt").into_response(),
    };

    let score_header = match ScoreHeader::from_decrypted(&score_data) {
        Some(d) => d,
        None => return (StatusCode::BAD_REQUEST, "error: score data < 2").into_response(),
    };

    let user = match authenticate_user(&state, &fields, &score_header.username).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    let beatmap = match repository::beatmap::fetch_by_md5(&state.db, &score_header.map_md5).await {
        Ok(Some(beatmap)) => beatmap,
        _ => return (StatusCode::BAD_REQUEST, "error: beatmap").into_response(),
    };

    let mut score = match Score::from_submission(&score_data[2..]) {
        Some(score) => score,
        None => return (StatusCode::BAD_REQUEST, "error: score").into_response(),
    };

    let _ = repository::user::update_latest_activity(&state.db, user.id).await;

    // idea: maybe, just maybe i could create another service that
    //       handles score validation with also player validation?
    //       would be fun, but for now, i will (?) complete this first.
    // ref: https://github.com/remeliah/meat-my-beat-i/blob/0121e875e142dbb7278ca4b171dd8c1095e26fb0/app/api/domains/osu.py#L719-L769
    //      https://github.com/remeliah/meat-my-beat-i/blob/main/app/usecases/ac.py

    score.acc = calculate_accuracy(&score);

    if let Ok(true) = ensure_local_osu_file(&state.config.omajinai, &beatmap).await {
        (score.pp, _) =
            calculate_score_performance(&state.config.omajinai, &score, beatmap.id).await;

        if score.passed() {
            if let Ok(Some(prev_best)) = calculate_status(&state.db, &mut score).await {
                let _ = repository::score::update_status(&state.db, prev_best.id, prev_best.status)
                    .await;
            }
        } else {
            score.status = SubmissionStatus::Failed.as_i32();
        }
    }

    score.time_elapsed = if score.passed() { fields.score_time } else { fields.fail_time };

    // todo

    (StatusCode::OK, "ok").into_response()
}
