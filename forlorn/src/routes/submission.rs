use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use webhook::{Author, Embed, Footer, Thumbnail, Webhook};

use crate::constants::{Grade, RankedStatus, SubmissionStatus};
use crate::dto::submission::{ScoreHeader, ScoreSubmission};
use crate::infrastructure::redis::publish::{announce, refresh_stats, restrict};
use crate::models::Score;
use crate::models::User;
use crate::repository;
use crate::state::AppState;
use crate::usecases::beatmap::{ensure_local_osu_file, increment_playcount};
use crate::usecases::password::verify_password;
use crate::usecases::score::{
    bind_cheat_values, calculate_accuracy, calculate_placement, calculate_score_performance,
    calculate_status, decrypt_score_data, update_any_preexisting_personal_best,
    validate_cheat_values,
};
use crate::usecases::stats::{get_computed_playtime, recalculate};
use crate::utils::{build_submission_charts, fmt_f, fmt_n};

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

async fn parse_typed_multipart(multipart: &mut Multipart) -> Result<ScoreSubmission, Response> {
    let mut score_data_b64: Option<Vec<u8>> = None;
    let mut replay_file: Option<Vec<u8>> = None;

    // HACK: to count how many "score" fields are there
    //       because osu! client sends both score data and replay file
    //       with the same field name "score" (pepy why)
    let mut score_count = 0;

    let mut fields: HashMap<String, Bytes> = HashMap::new();

    while let Some(field) = multipart.next_field().await.ok().flatten() {
        let name = field.name().map(|s| s.to_owned()).unwrap_or_default();
        let content = field.bytes().await.unwrap_or_default();

        if name == "score" {
            if score_count == 0 {
                score_data_b64 = Some(content.to_vec());
            } else if score_count == 1 {
                replay_file = Some(content.to_vec());
            }

            score_count += 1;
            continue;
        }

        fields.insert(name, content);
    }

    let score_data_b64 = score_data_b64
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "error: score data").into_response())?;

    let replay_file = replay_file
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "error: replay file").into_response())?;

    let submission = ScoreSubmission {
        exited_out: fields
            .get("x")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        fail_time: fields
            .get("ft")
            .and_then(|b| String::from_utf8_lossy(b).parse().ok())
            .unwrap_or(0),
        visual_settings_b64: fields
            .get("fs")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        updated_beatmap_hash: fields
            .get("bmk")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        storyboard_md5: fields
            .get("sbk")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        iv_b64: fields
            .get("iv")
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "error: iv").into_response())?
            .clone(),
        unique_ids: fields
            .get("c1")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        score_time: fields
            .get("st")
            .and_then(|b| String::from_utf8_lossy(b).parse().ok())
            .unwrap_or(0),
        password_md5: fields
            .get("pass")
            .map(|b| String::from_utf8_lossy(b).to_string())
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "error: password").into_response())?,
        osu_version: fields
            .get("osuver")
            .map(|b| String::from_utf8_lossy(b).to_string())
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "error: osu version").into_response())?,
        client_hash_b64: fields
            .get("s")
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "error: client hash").into_response())?
            .clone(),
        aim_value: fields
            .get("acval")
            .and_then(|b| String::from_utf8_lossy(b).parse().ok())
            .unwrap_or(0),
        ar_value: fields
            .get("arval")
            .and_then(|b| String::from_utf8_lossy(b).parse().ok())
            .unwrap_or(0.0),
        aim: fields
            .get("ac")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        arc: fields
            .get("ar")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        hdr: fields
            .get("hdrem")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        cs: fields
            .get("cs")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        tw: fields
            .get("tw")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        twval: fields
            .get("twval")
            .and_then(|b| String::from_utf8_lossy(b).parse().ok())
            .unwrap_or(0.0),
        refx: fields
            .get("refx")
            .map(|b| String::from_utf8_lossy(b).to_string()),
        score_data_b64,
        replay_file,
    };

    Ok(submission)
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

    let submission = match parse_typed_multipart(&mut multipart).await {
        Ok(d) => d,
        Err(response) => return response,
    };

    let (score_data, _) = match decrypt_score_data(
        &submission.score_data_b64,
        &submission.client_hash_b64,
        &submission.iv_b64,
        &submission.osu_version,
    ) {
        Ok((v, c)) => (v, c),
        Err(_) => return (StatusCode::BAD_REQUEST, "error: decrypt").into_response(),
    };

    let score_header = match ScoreHeader::from_decrypted(&score_data) {
        Some(d) => d,
        None => return (StatusCode::BAD_REQUEST, "error: score data < 2").into_response(),
    };

    let user =
        match authenticate_user(&state, &submission.password_md5, &score_header.username).await {
            Ok(user) => user,
            Err(response) => return response,
        };

    let mut beatmap =
        match repository::beatmap::fetch_by_md5(&state.db, &score_header.map_md5).await {
            Ok(Some(beatmap)) => beatmap,
            _ => return (StatusCode::BAD_REQUEST, "error: beatmap").into_response(),
        };

    let mut score = match Score::from_submission(&score_data[2..]) {
        Some(score) => score,
        None => return (StatusCode::BAD_REQUEST, "error: score").into_response(),
    };

    // What the fuck.
    // i genuinely forgot about score.map_md5 & score.userid and i somehow
    // used that on some usecases functions without realizing its not "setted" yet
    // this is actually a lesson for me to not name fields poorly and misunderstanding ;d
    // TODO: REMOVE
    score.map_md5 = score_header.map_md5;
    score.userid = user.id;

    let _ = repository::user::update_latest_activity(&state.db, user.id).await;

    // idea: maybe, just maybe i could create another service that
    //       handles score validation with also player validation?
    //       would be fun, but for now, i will (?) complete this first.
    // ref: https://github.com/remeliah/meat-my-beat-i/blob/0121e875e142dbb7278ca4b171dd8c1095e26fb0/app/api/domains/osu.py#L719-L769
    //      https://github.com/remeliah/meat-my-beat-i/blob/main/app/usecases/ac.py

    bind_cheat_values(&mut score, &submission);

    if submission.refx() && !validate_cheat_values(&score) {
        let webhook = Webhook::new(&state.config.webhook.debug).content(format!(
            "[{}] <{} ({})> Overcheat? (malformed cheat value) [ac={}|tw={}|cs={}]",
            score.mode().as_str(),
            user.name,
            user.id,
            score.aim_correction_value,
            score.timewarp_value,
            score.uses_cs_changer
        ));

        tracing::warn!(
            "[{}] <{} ({})> submitted a malformed cheat value [ac={}|tw={}|cs={}]",
            score.mode().as_str(),
            user.name,
            user.id,
            score.aim_correction_value,
            score.timewarp_value,
            score.uses_cs_changer
        );

        let _ = webhook.post().await;

        // NOTE: it's not a good idea to return here,
        //       we let them submit since its possibly their client's submission error.
        //       or theres a big flaw on the client that ano (me) need to fix
        //       god i dont want to open up rider
    }

    score.acc = calculate_accuracy(&score);

    if let Ok(true) = ensure_local_osu_file(&state.config.omajinai, &beatmap).await {
        (score.pp, score.stars) =
            calculate_score_performance(&state.config.omajinai, &score, beatmap.id).await;

        if score.passed {
            if let Ok(Some(prev_best)) = calculate_status(&state.db, &mut score).await {
                let _ = repository::score::update_status(&state.db, prev_best.id, prev_best.status)
                    .await;
            }

            if beatmap.status != RankedStatus::Pending.as_i32() {
                score.rank = calculate_placement(&state.db, &score).await;
            }
        } else {
            score.status = SubmissionStatus::Failed.as_i32();
        }
    }

    score.time_elapsed = if score.passed { submission.score_time } else { submission.fail_time };

    if score.status == SubmissionStatus::Best.as_i32() {
        if beatmap.has_leaderboard() && score.rank == 1 && !user.restricted() {
            let prev_holder = repository::user::fetch_prev_n1(&state.db, &score)
                .await
                .ok()
                .flatten();

            // TODO: move these to usecases
            let desc = format!(
                "{} ▸ {}pp ▸ {}\n{:.2}% ▸ [{}/{}/{}/{}x] ▸ {}/{}x ▸ {}",
                score.grade().discord_emoji(),
                fmt_f(score.pp),    // formatted to match python
                fmt_n(score.score), // formatted to match python
                score.acc,
                score.n300,
                score.n100,
                score.n50,
                score.nmiss,
                score.max_combo,
                beatmap.max_combo,
                score.mods().as_str()
            );

            #[allow(clippy::uninlined_format_args)]
            let content: String = if let Some((prev_id, prev_name)) = prev_holder {
                format!(
                    "\n\npreviously held by [{}](https://remeliah.cyou/u/{})",
                    prev_name, prev_id
                )
            } else {
                String::new()
            };

            // TODO: pp record announce

            let embed = Embed::new()
                .title(format!("{} - {:.2}★", beatmap.full_name(), score.stars))
                .url(beatmap.url())
                .description(desc)
                .color(2829617) // gray
                .author(Author::new().name(format!("set a new #1 worth {:.2}pp", score.pp)))
                .thumbnail(Thumbnail::new().url(format!(
                    "https://assets.ppy.sh/beatmaps/{}/covers/card.jpg",
                    beatmap.set_id
                )))
                .footer(Footer::new(format!("{} | forlorn", score.mode().as_str()))); // trole

            let webhook = Webhook::new(&state.config.webhook.score)
                .username(&user.name)
                .content(content)
                .avatar_url(format!("https://a.remeliah.cyou/{}", user.id))
                .add_embed(embed);

            let _ = webhook.post().await;
        }

        update_any_preexisting_personal_best(&state.db, &score).await;
    }

    score.id = match repository::score::insert(&state.db, &score, &beatmap).await {
        Ok(id) => id,
        _ => return (StatusCode::INTERNAL_SERVER_ERROR, "error: insert").into_response(),
    };

    if score.passed {
        if score.rank == 1 && beatmap.has_leaderboard() {
            let _ = announce::announce(&state.redis, score.id).await;
        }

        const MIN_REPLAY_SIZE: usize = 24;

        if submission.replay_file.len() >= MIN_REPLAY_SIZE {
            let replay_path = state.config.replay_path.join(format!("{}.osr", score.id));

            if (tokio::fs::write(&replay_path, &submission.replay_file).await).is_err() {
                // NOTE: not returning here since it would break submission (duh)
            }
        } else {
            let _ = restrict::restrict(&state.redis, user.id, "score submitter?").await;
        }
    }

    // update player & beatmap stats
    let mut stats =
        match repository::stats::fetch_by_user_mode(&state.db, &state.redis, user.id, score.mode)
            .await
        {
            Ok(Some(stats)) => stats,
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, "error: stats").into_response(),
        };

    let prev_stats = stats.clone();

    stats.playtime += get_computed_playtime(&score, &beatmap).await;
    stats.plays += 1;
    stats.tscore += score.score as u64;
    stats.total_hits += score.n300 as u32 + score.n100 as u32 + score.n50 as u32;

    if score.mode().ngeki_nkatu() {
        stats.total_hits += score.ngeki as u32 + score.nkatu as u32;
    }

    let mut stats_updates = HashMap::new();
    stats_updates.insert("plays", stats.plays);
    stats_updates.insert("playtime", stats.playtime);
    stats_updates.insert("tscore", stats.tscore as u32);
    stats_updates.insert("total_hits", stats.total_hits);

    if score.passed && beatmap.has_leaderboard() {
        if score.max_combo as u32 > stats.max_combo {
            stats.max_combo = score.max_combo as u32;
        }

        if beatmap.awards_ranked_pp() && score.status == SubmissionStatus::Best.as_i32() {
            // TODO: i think i need to place prev_best on score models since i frequently call this
            let prev_best =
                repository::score::fetch_best(&state.db, user.id, &beatmap.md5, score.mode)
                    .await
                    .ok()
                    .flatten();

            let mut additional_rscore = score.score;
            if let Some(ref pb) = prev_best {
                additional_rscore -= pb.score;

                if score.grade() != pb.grade() {
                    if score.grade() >= Grade::A {
                        stats.increment_grade(score.grade());
                    }

                    if pb.grade() >= Grade::A {
                        stats.decrement_grade(pb.grade());
                    }
                }
            } else if score.grade() >= Grade::A {
                stats.increment_grade(score.grade());
            }

            stats.rscore += additional_rscore as u64;

            if (recalculate(&state.db, &mut stats).await).is_err() {
                return (StatusCode::INTERNAL_SERVER_ERROR, "error: recalculate").into_response();
            }

            // TODO: send notification to player

            if let Ok(new_rank) = repository::stats::update_rank(
                &state.redis,
                &stats,
                &user.country,
                user.restricted(),
            )
            .await
            {
                stats.rank = new_rank as i32;
            }
        }
    }

    if (repository::stats::save(&state.db, &stats).await).is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "error: save stats").into_response();
    }

    if !user.restricted() {
        let _ = refresh_stats::refresh_stats(&state.redis, user.id).await;
        let _ = increment_playcount(&state.db, &mut beatmap, score.passed).await;
    }

    tracing::info!(
        "[{}] {} submitted a score! ({}, {}pp | {}pp)",
        score.mode().as_str(),
        user.name,
        score.status().as_str(),
        score.pp.round(),
        stats.pp,
    );

    let charts = build_submission_charts(&score, &beatmap, &stats, &prev_stats, &state).await;

    (StatusCode::OK, charts.into_bytes()).into_response()
}
