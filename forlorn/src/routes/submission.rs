use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use webhook::Webhook;

use crate::{
    constants::{Grade, RankedStatus, SubmissionStatus},
    dto::submission::{ScoreHeader, ScoreSubmission},
    infrastructure::redis::publish::{announce, notify, refresh_stats, restrict},
    models::{Score, User},
    repository,
    state::AppState,
    usecases::{
        beatmap::{ensure_local_osu_file, increment_playcount},
        password::verify_password,
        score::{
            calculate_accuracy, calculate_performance, calculate_placement, calculate_status,
            calculate_xp, consume_cheat_values, decrypt_score_data, first_place_webhook,
            update_any_preexisting_personal_best, validate_cheat_values,
        },
        stats::{get_computed_playtime, recalculate},
    },
    utils::{build_submission, build_submission_charts},
};

const REFX_CURRENT_CLIENT_HASH: &str = "230cd99998f1a18dbc787612179bae0e";

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
    let mut score_fields = Vec::new();
    let mut fields: HashMap<String, Bytes> = HashMap::new();

    while let Some(field) = multipart.next_field().await.ok().flatten() {
        let name = field.name().unwrap_or_default().to_owned();
        let content = field.bytes().await.unwrap_or_default();

        if name == "score" {
            score_fields.push(content.to_vec());
        } else {
            fields.insert(name, content);
        }
    }

    let [score_data_b64, replay_file]: [Vec<u8>; 2] = score_fields
        .try_into()
        .map_err(|_| (StatusCode::OK, b"error: no").into_response())?;

    match build_submission(score_data_b64, replay_file, fields) {
        Some(submission) => Ok(submission),
        None => Err((StatusCode::OK, b"error: no").into_response()),
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
        return (StatusCode::OK, b"error: oldver").into_response();
    }

    let now = Instant::now();

    let submission = match parse_typed_multipart(&mut multipart).await {
        Ok(d) => d,
        Err(response) => return response,
    };

    let (score_data, osu_path_md5) = match decrypt_score_data(
        &submission.score_data_b64,
        &submission.client_hash_b64,
        &submission.iv_b64,
        &submission.osu_version,
    ) {
        Ok((v, c)) => (v, c),
        Err(_) => return (StatusCode::OK, b"error: no").into_response(),
    };

    let score_header = match ScoreHeader::from_decrypted(&score_data) {
        Some(d) => d,
        None => return (StatusCode::OK, b"error: no").into_response(),
    };

    let user =
        match authenticate_user(&state, &submission.password_md5, &score_header.username).await {
            Ok(user) => user,
            Err(response) => return response,
        };

    // NOTE: a small combat for the "refx" client
    //       this shouldn't even get passed since bancho already handles this?
    //       but for extra safety, maybe i should restrict them too?
    //       since they most likely spoofed `GameBase.ClientHash`.
    if submission.refx() && osu_path_md5 != REFX_CURRENT_CLIENT_HASH {
        tracing::warn!(
            "{} submitted a score in outdated/modified re;fx client! ({} != {})",
            user.name(),
            osu_path_md5,
            REFX_CURRENT_CLIENT_HASH,
        );

        {
            let r = state.redis.clone();
            tokio::spawn(async move {
                let _ = notify::notify(&r, user.id, "Please update your client!").await;
            });
        }

        return (StatusCode::OK, b"error: no").into_response();
    }

    let beatmap =
        match repository::beatmap::fetch_by_md5(&state.config, &state.db, &score_header.map_md5)
            .await
        {
            Ok(Some(beatmap)) => beatmap,
            _ => return (StatusCode::OK, b"error: beatmap").into_response(),
        };

    let mut score = match Score::from_submission(&score_data[2..], score_header.map_md5, user.id) {
        Some(score) => score,
        None => return (StatusCode::OK, b"error: no").into_response(),
    };

    score.mode = score.mode().as_i32();
    score.acc = calculate_accuracy(&score);
    score.quit = submission.exited_out();
    consume_cheat_values(&mut score, &submission);

    // always update last activity no matter what
    {
        let d = state.db.clone();
        tokio::spawn(async move {
            let _ = repository::user::update_latest_activity(&d, user.id).await;
        });
    }

    if submission.refx() && !validate_cheat_values(&score) {
        let webhook = Webhook::new(&state.config.webhook.debug).content(format!(
            "[{}] {} Overcheat? (malformed cheat value) [ac={}|tw={}|cs={}]",
            score.mode().as_str(),
            user.name(),
            score.aim_correction_value,
            score.timewarp_value,
            score.uses_cs_changer
        ));

        tracing::warn!(
            "[{}] {} submitted a malformed cheat value [ac={}|tw={}|cs={}]",
            score.mode().as_str(),
            user.name(),
            score.aim_correction_value,
            score.timewarp_value,
            score.uses_cs_changer
        );

        tokio::spawn(async move {
            let _ = webhook.post().await;
        });

        // NOTE: it's not a good idea to return here,
        //       we let them submit since its possibly their client's submission error.
        //       or theres a big flaw on the client that ano (me) need to fix
        //       god i dont want to open up rider
    }

    match ensure_local_osu_file(
        state.storage.beatmap_file(beatmap.id),
        &state.config.omajinai,
        &beatmap,
    )
    .await
    {
        Ok(true) => {},
        _ => {
            return (StatusCode::OK, b"error: no").into_response();
        },
    }

    let _submission_lock_ = match state
        .score_locks
        .lock(
            format!("refx:score_submission:{}", score.online_checksum).as_bytes(),
            Duration::from_secs(15),
        )
        .await
    {
        Ok(lock) => lock,
        _ => {
            tracing::warn!(
                "failed to acquire submission lock for {}",
                score.online_checksum,
            );

            // empty response so we can tell them to retry
            return (StatusCode::OK).into_response();
        },
    };

    let charts = {
        if let Ok(Some(_)) =
            repository::score::fetch_by_online_checksum(&state.db, &score.online_checksum).await
        {
            tracing::warn!(
                "duplicate score submission detected for user: {}",
                user.name
            );

            return (StatusCode::OK, b"error: no").into_response();
        }

        (score.pp, score.stars, score.hypothetical_pp) = calculate_performance(
            &state.config.omajinai,
            beatmap.id,
            score.mode,
            score.mods,
            score.max_combo,
            score.acc,
            score.nmiss,
            score.score,
        )
        .await;

        if score.passed {
            if let Ok(Some(prev_best)) = calculate_status(&state.db, &mut score).await {
                let _ = repository::score::update_status(&state.db, prev_best.id, prev_best.status)
                    .await;
            }

            if beatmap.status != RankedStatus::Pending.as_i32() {
                score.rank = calculate_placement(&state.db, &score).await;
            }
        } else if score.quit {
            score.status = SubmissionStatus::Quit.as_i32();
        } else {
            score.status = SubmissionStatus::Failed.as_i32();
        }

        score.time_elapsed =
            if score.passed { submission.score_time } else { submission.fail_time };

        score.xp = calculate_xp(&score, &beatmap);

        if score.status == SubmissionStatus::Best.as_i32() {
            if beatmap.has_leaderboard() && score.rank == 1 && !user.restricted() {
                let prev_holder = repository::user::fetch_prev_n1(&state.db, &score)
                    .await
                    .ok()
                    .flatten();

                let webhook = first_place_webhook(
                    &user,
                    &score,
                    &beatmap,
                    &state.config.webhook.score,
                    prev_holder,
                );

                tokio::spawn(async move {
                    let _ = webhook.post().await;
                });
            }

            update_any_preexisting_personal_best(&state.db, &score).await;
        }

        score.id = match repository::score::insert(&state.db, &score, &beatmap).await {
            Ok(id) => id,
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, b"error: no").into_response(),
        };

        if score.passed {
            if score.rank == 1
                && beatmap.has_leaderboard()
                && !user.restricted()
                && score.status == SubmissionStatus::Best.as_i32()
            {
                let r = state.redis.clone();
                tokio::spawn(async move {
                    let _ = announce::announce(&r, score.id).await;
                });
            }

            const MIN_REPLAY_SIZE: usize = 24;

            if submission.replay_file.len() >= MIN_REPLAY_SIZE {
                let replay_path = state.storage.replay_file(score.id);

                tokio::spawn(async move {
                    if (tokio::fs::write(&replay_path, &submission.replay_file).await).is_err() {
                        tracing::warn!("failed to save replay! ({})", score.id)
                    }
                });
            } else {
                let r = state.redis.clone();
                tokio::spawn(async move {
                    let _ = restrict::restrict(&r, user.id, "score submitter?").await;
                });
            }

            if let (true, Some(threshold)) = score.check_pp_cap(&user)
                && beatmap.awards_ranked_pp()
            {
                tracing::warn!(
                    "[{}] {} restricted for suspicious pp gain ({}pp > {}pp)",
                    score.mode().as_str(),
                    user.name(),
                    score.pp.round(),
                    threshold,
                );

                {
                    let r = state.redis.clone();
                    let _ = restrict::restrict(
                        &r,
                        user.id,
                        &format!("suspicious pp gain ({}pp)", score.pp.round(),),
                    )
                    .await;
                }
            }
        }

        // update player & beatmap stats
        let mut stats = match repository::stats::fetch_by_user_mode(
            &state.db,
            &state.redis,
            user.id,
            score.mode,
        )
        .await
        {
            Ok(Some(stats)) => stats,
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, b"error: no").into_response(),
        };

        let prev_stats = stats.clone();

        stats.playtime += get_computed_playtime(&score, &beatmap);
        stats.plays += 1;
        stats.tscore += score.score as u64;
        stats.total_hits += score.total_hits();
        stats.xp += score.xp.round() as i32;

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

                if score.pp > 0.0 && (recalculate(&state.db, &mut stats).await).is_err() {
                    return (StatusCode::INTERNAL_SERVER_ERROR, b"error: no").into_response();
                }

                // casts as i32 so "let there be negative"
                // TODO: is this really a good name
                let pp_lost_gained = stats.pp as i32 - prev_stats.pp as i32;
                let mut notify_message =
                    format!("You achieved #{}!, ({:.2}pp)", score.rank, score.pp);

                if pp_lost_gained > 0 {
                    notify_message += &format!(" and gained {pp_lost_gained:.2}pp!");
                } else if pp_lost_gained < 0 {
                    notify_message += &format!(" but lost {:.2}pp!", pp_lost_gained.abs());
                }

                {
                    let r = state.redis.clone();
                    tokio::spawn(async move {
                        let _ = notify::notify(&r, user.id, &notify_message).await;
                    });
                }

                if let Ok(new_rank) = repository::stats::update_rank(
                    &state.redis,
                    &stats,
                    &user.country,
                    user.restricted(),
                )
                .await
                {
                    stats.rank = new_rank;
                }
            }
        }

        if (repository::stats::save(&state.db, &stats).await).is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, b"error: no").into_response();
        }

        if !user.restricted() {
            let r = state.redis.clone();
            tokio::spawn(async move {
                let _ = refresh_stats::refresh_stats(&r, user.id).await;
            });
        }

        if !user.restricted() {
            let d = state.db.clone();
            let mut b = beatmap.clone();
            tokio::spawn(async move {
                let _ = increment_playcount(&d, &mut b, score.passed).await;
            });
        }

        let done = now.elapsed();

        tracing::info!(
            "[{}] {} submitted a score! ({}, {}pp | {}pp) in {}ms.",
            score.mode().as_str(),
            user.name(),
            score.status().as_str(),
            score.pp.round(),
            stats.pp,
            done.as_millis(),
        );

        build_submission_charts(&score, &beatmap, &stats, &prev_stats, &state).await
    };

    (StatusCode::OK, charts.into_bytes()).into_response()
}
