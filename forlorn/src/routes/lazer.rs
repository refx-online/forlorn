use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use chrono::{DateTime, Utc};

use crate::{
    constants::{LeaderboardType, RankedStatus, SubmissionStatus},
    infrastructure::{
        beatmap_service,
        redis::publish::{announce, refresh_stats, score},
    },
    models::Score,
    repository,
    state::AppState,
    usecases::{
        beatmap::increment_playcount,
        score::{
            calculate_performance, calculate_placement, calculate_status, calculate_xp,
            update_any_preexisting_personal_best,
        },
    },
};

/// A lazer score submission, already validated and normalised by speedforce.
///
/// lazer sends `SoloScoreInfo` JSON; speedforce parses it, derives the effective
/// mode from the mods, and forwards the result here so the write goes through the
/// same usecase chain stable uses. That keeps PB rejection, placement, xp, pp,
/// leaderboards and redis events in one place -- duplicating them would let lazer
/// and stable stats drift apart.
#[derive(Deserialize)]
pub struct LazerScoreSubmission {
    pub user_id: i32,
    /// lazer's own id for the beatmap, used for the omajinai pp call
    pub beatmap_id: i32,
    pub map_md5: String,
    /// dense bancho mode id, derived by speedforce from lazer's ruleset + mods
    pub mode: i32,
    pub mods: i32,
    pub passed: bool,
    pub rank: String,
    pub total_score: i64,
    pub accuracy: f64,
    pub max_combo: i32,
    pub build_id: Option<i64>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub counts: LazerHitCounts,
    /// stored verbatim in lazer_scores.statistics_json
    pub statistics: serde_json::Value,
    pub mods_list: Vec<LazerMod>,
}

#[derive(Deserialize)]
pub struct LazerHitCounts {
    pub n300: i32,
    pub n100: i32,
    pub n50: i32,
    pub nmiss: i32,
    pub ngeki: i32,
    pub nkatu: i32,
}

#[derive(Deserialize, Serialize)]
pub struct LazerMod {
    pub acronym: String,
    #[serde(default)]
    pub settings: serde_json::Value,
}

#[derive(serde::Serialize)]
pub struct LazerScoreAccepted {
    pub score_id: u64,
    pub position: u32,
    pub ranked: bool,
    pub pp: f32,
    pub status: i32,
}

/// Guarded by a shared secret rather than a user token: only speedforce should
/// call this, and it has already authenticated the user.
fn authorised(headers: &HeaderMap, expected: &str) -> bool {
    if expected.is_empty() {
        // refuse to run unauthenticated rather than defaulting to open
        return false;
    }
    headers
        .get("x-internal-token")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == expected)
        .unwrap_or(false)
}

pub async fn submit_lazer_score(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(sub): Json<LazerScoreSubmission>,
) -> Response {
    if !authorised(&headers, &state.config.lazer_internal_token) {
        return (StatusCode::UNAUTHORIZED, "forbidden").into_response();
    }

    let user = match repository::user::fetch_by_id(&state.db, &sub.user_id).await {
        Ok(Some(u)) => u,
        _ => return (StatusCode::NOT_FOUND, "unknown user").into_response(),
    };

    let beatmap = match beatmap_service::fetch_beatmap_with_lifecycle(
        &state.config.omajinai.beatmap_service_url,
        &sub.map_md5,
        None,
    )
    .await
    {
        Ok(beatmap_service::BeatmapLifecycleStatus::Available(b)) => *b,
        Ok(beatmap_service::BeatmapLifecycleStatus::Unsubmitted) => {
            return (StatusCode::UNPROCESSABLE_ENTITY, "beatmap unsubmitted").into_response()
        }
        Ok(beatmap_service::BeatmapLifecycleStatus::UpdateRequired) => {
            return (StatusCode::UNPROCESSABLE_ENTITY, "beatmap update required").into_response()
        }
        Err(e) => {
            tracing::warn!("lazer submit: beatmap service error: {e}");
            return (StatusCode::BAD_GATEWAY, "beatmap service error").into_response();
        }
    };

    let now = Utc::now();
    let played = sub
        .started_at
        .or(sub.ended_at)
        .unwrap_or_else(|| now - chrono::Duration::seconds(60));

    // lazer's own accuracy is used verbatim: recomputing from the six legacy
    // counters would throw away the precision lazer's per-object judgement has.
    let mut s = Score {
        id: 0,
        map_md5: sub.map_md5.clone(),
        score: sub.total_score as i32,
        xp: 0.0,
        pp: 0.0,
        acc: sub.accuracy as f32,
        max_combo: sub.max_combo,
        mods: sub.mods,
        n300: sub.counts.n300,
        n100: sub.counts.n100,
        n50: sub.counts.n50,
        nmiss: sub.counts.nmiss,
        ngeki: sub.counts.ngeki,
        nkatu: sub.counts.nkatu,
        grade: sub.rank.clone(),
        // provisional; calculate_status() below settles it against the
        // player's previous best once pp is known
        status: SubmissionStatus::Best.as_i32(),
        mode: sub.mode,
        play_time: DateTime::<Utc>::from(played),
        time_elapsed: sub
            .started_at
            .zip(sub.ended_at)
            .map(|(a, b)| (b - a).num_seconds() as i32)
            .unwrap_or_default(),
        // lazer doesn't send osu!'s client_flags byte
        client_flags: 0,
        userid: sub.user_id,
        perfect: sub.counts.ngeki > 0,
        online_checksum: String::new(),
        // lazer reports no aim/ar/tw/cs changer values; these are the "unused"
        // sentinels the schema expects.
        aim_correction_value: -1,
        ar_changer_value: -1.0,
        uses_aim_correction: false,
        uses_ar_changer: false,
        uses_cs_changer: false,
        uses_timewarp: false,
        timewarp_value: -1.0,
        uses_hd_remover: false,
        aim_assist_type: 0,
        clock_rate: 1.0,
        maple_values: None,
        pinned: false,
        rank: 0,
        hypothetical_pp: 0.0,
        stars: 0.0,
        passed: sub.passed,
        quit: false,
    };

    let _ = state.metrics.incr("score.submitted", ["status:lazer"]);

    // Read before calculate_status, which demotes the player's previous best to
    // Submitted -- a lookup afterwards would miss it and award the full new
    // score as the rscore delta instead of just the improvement.
    let prev_best = repository::score::fetch_best(&state.db, user.id, &beatmap.md5, s.mode)
        .await
        .ok()
        .flatten();

    // lazer's hit counts are per-object; pass them through as-is. nmiss has no
    // parameter of its own (omajinai derives misses from the rest).
    (s.pp, s.stars, s.hypothetical_pp) = match calculate_performance(
        &state.config.omajinai,
        sub.beatmap_id,
        s.mode,
        s.mods,
        s.max_combo,
        s.acc,
        s.nmiss,
        s.score,
        s.clock_rate,
        Some(s.n300),
        Some(s.n100),
        Some(s.n50),
        Some(s.ngeki),
        Some(s.nkatu),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("lazer submit: pp calculation failed: {e:?}");
            (0.0, 0.0, 0.0)
        }
    };

    if s.passed {
        if let Ok(Some(prev_best)) = calculate_status(&state.db, &mut s).await {
            let _ = repository::score::update_status(&state.db, prev_best.id, prev_best.status).await;
        }

        if beatmap.mode_status(s.mode) != RankedStatus::Pending.as_i32() {
            s.rank = calculate_placement(&state.db, &s).await;
        }
    } else {
        s.status = SubmissionStatus::Failed.as_i32();
    }

    s.xp = calculate_xp(&s, &beatmap);

    if s.status == SubmissionStatus::Best.as_i32() {
        update_any_preexisting_personal_best(&state.db, &s).await;
    }

    s.id = match repository::score::insert(&state.db, &s, &beatmap).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("lazer submit: score insert failed for {}: {e:?}", user.name());
            return (StatusCode::INTERNAL_SERVER_ERROR, "score insert failed").into_response();
        }
    };

    // Aggregate stats + leaderboards. Same call stable's submission route makes,
    // so `stats` rows and the bancho:leaderboard:* ZSETs stay in agreement
    // whichever client submitted.
    match crate::usecases::stats::apply_score(&state.db, &state.redis, &user, &s, &beatmap, prev_best.as_ref())
        .await
    {
        Ok(applied) => {
            tracing::debug!(
                "lazer stats: {} mode {} -> #{} {:.0}pp",
                user.name(),
                applied.stats.mode,
                applied.stats.rank,
                applied.stats.pp
            );
        }
        Err(e) => {
            // the score itself is already durable; log loudly but still ack, so a
            // stats failure never makes lazer retry a submission it already took.
            tracing::error!(
                "lazer stats update failed for user {}: {e:?}",
                user.name()
            );
        }
    }

    // lazer-only detail that the stable `scores` row cannot represent
    if let Err(e) = repository::score::insert_lazer_details(
        &state.db,
        s.id,
        sub.build_id.unwrap_or(0),
        played,
        sub.ended_at.unwrap_or(now),
        &sub.statistics,
        &sub.mods_list,
    )
    .await
    {
        tracing::warn!("lazer_scores insert failed for score {}: {e:?}", s.id);
    }

    if s.passed && s.rank == 1
        && beatmap.has_leaderboard_in(s.mode)
        && !user.restricted()
        && s.status == SubmissionStatus::Best.as_i32()
    {
        let r = state.redis.clone();
        tokio::spawn(async move {
            let _ = announce::announce(&r, s.id).await;
        });
    }

    if !user.restricted() {
        let r = state.redis.clone();
        let uid = user.id;
        tokio::spawn(async move {
            let _ = refresh_stats::refresh_stats(&r, uid).await;
        });

        let d = state.db.clone();
        let mut b = beatmap.clone();
        tokio::spawn(async move {
            let _ = increment_playcount(&d, &mut b, s.passed).await;
        });
    }

    {
        let r = state.redis.clone();
        tokio::spawn(async move {
            let _ = score::score_submitted(&r, s.id).await;
        });
    }

    tracing::info!(
        "[{}] {} submitted a lazer score! ({} | {}pp) via speedforce.",
        s.mode().as_str(),
        user.name(),
        s.status().as_str(),
        s.pp.round(),
    );

    Json(LazerScoreAccepted {
        score_id: s.id,
        position: s.rank,
        ranked: s.status == SubmissionStatus::Best.as_i32(),
        pp: s.pp,
        status: s.status,
    })
    .into_response()
}

/// One leaderboard row, shaped for speedforce. Deliberately not the whole
/// `LeaderboardScore`: the lazer client only needs these, and keeping the
/// surface small means the two sides can't drift on internal columns.
#[derive(serde::Serialize)]
pub struct LazerLeaderboardScore {
    pub id: u64,
    pub user_id: i32,
    pub username: String,
    pub score: i32,
    pub accuracy: f32,
    pub grade: String,
    pub country_code: Option<String>,
    pub max_combo: i32,
    pub mods: i32,
    pub n300: i32,
    pub n100: i32,
    pub n50: i32,
    pub nmiss: i32,
    pub ngeki: i32,
    pub nkatu: i32,
    pub perfect: bool,
    pub play_time: i64,
    pub pp: f64,
}

#[derive(serde::Serialize)]
pub struct LazerLeaderboard {
    pub scores: Vec<LazerLeaderboardScore>,
}

/// lazer's leaderboard read path.
///
/// Reuses `repository::leaderboard::fetch_leaderboard_scores` -- the same query
/// the stable `osu-get-leaderboard.php` route runs -- so both clients see one
/// leaderboard: lazer PBs and stable PBs are interleaved and ranked together,
/// and mods expansion / friend+country filtering stay in one place.
pub async fn get_lazer_leaderboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(map_md5): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<LazerLeaderboardQuery>,
) -> Response {
    if !authorised(&headers, &state.config.lazer_internal_token) {
        return (StatusCode::UNAUTHORIZED, "forbidden").into_response();
    }

    let beatmap = match beatmap_service::fetch_beatmap_with_lifecycle(
        &state.config.omajinai.beatmap_service_url,
        &map_md5,
        None,
    )
    .await
    {
        Ok(beatmap_service::BeatmapLifecycleStatus::Available(b)) => *b,
        _ => return (StatusCode::NOT_FOUND, "unknown beatmap").into_response(),
    };

    // lazer only asks for rulesets 0-7; anything else is a malformed request.
    let mode = params.mode.unwrap_or(beatmap.mode as i32);
    if !(0..8).contains(&mode) {
        return (StatusCode::BAD_REQUEST, "bad ruleset").into_response();
    }

    let user_id = params.user_id.unwrap_or_default();
    let rows = match repository::leaderboard::fetch_leaderboard_scores(
        &state.db,
        &map_md5,
        mode,
        user_id,
        LeaderboardType::Top,
        None,
        None,
        None,
        "pp",
        // true selects the extra column set that includes `s.score`. lazer's
        // leaderboard needs the real score per row, and the refx anti-cheat
        // columns that come with it are simply unused here.
        true,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("lazer leaderboard query failed: {e:?}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "query failed").into_response();
        }
    };

    let scores = rows
        .into_iter()
        .map(|r| LazerLeaderboardScore {
            id: r.id,
            user_id: r.userid,
            // the query prefixes clan tags for stable; lazer shows them itself
            username: r.name.rsplit_once("] ").map_or(r.name.as_str(), |(_, n)| n).to_string(),
            score: r.actual_score.unwrap_or(0),
            accuracy: r.accuracy,
            grade: r.grade,
            country_code: r.country_code,
            max_combo: r.max_combo,
            mods: r.mods,
            n300: r.n300,
            n100: r.n100,
            n50: r.n50,
            nmiss: r.nmiss,
            ngeki: r.ngeki,
            nkatu: r.nkatu,
            perfect: r.perfect,
            play_time: r.play_time,
            pp: r.preferred_metric,
        })
        .collect();

    (StatusCode::OK, Json(LazerLeaderboard { scores })).into_response()
}

#[derive(serde::Deserialize)]
pub struct LazerLeaderboardQuery {
    pub mode: Option<i32>,
    /// the requesting user, used for the personal-best highlight
    pub user_id: Option<i32>,
}

/// lazer frames its replay upload as int32 legacyScoreId + raw .osr bytes.
/// speedforce already validates and strips that prefix before forwarding, so
/// whatever arrives here is the bare replay and is stored as-is like a stable
/// one -- stripping again here silently corrupted every upload by 4 bytes.

#[derive(serde::Serialize)]
pub struct LazerReplayUpload {
    pub score_id: u64,
    pub bytes: usize,
}

/// Store a replay lazer uploaded for an already-accepted score.
pub async fn put_lazer_replay(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(score_id): axum::extract::Path<u64>,
    body: axum::body::Bytes,
) -> Response {
    if !authorised(&headers, &state.config.lazer_internal_token) {
        return (StatusCode::UNAUTHORIZED, "forbidden").into_response();
    }

    if repository::score::fetch_by_id(&state.db, score_id).await.ok().flatten().is_none() {
        return (StatusCode::NOT_FOUND, "unknown score").into_response();
    }

    let raw: &[u8] = &body;

    if raw.is_empty() {
        return (StatusCode::BAD_REQUEST, "empty replay").into_response();
    }

    if let Err(e) = state.storage.save_replay(score_id, raw).await {
        tracing::error!("lazer replay save failed for {score_id}: {e:?}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "replay save failed").into_response();
    }

    (StatusCode::OK, Json(LazerReplayUpload { score_id, bytes: raw.len() })).into_response()
}

/// Read a stored replay back out for lazer's download endpoint.
pub async fn get_lazer_replay(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(score_id): axum::extract::Path<u64>,
) -> Response {
    if !authorised(&headers, &state.config.lazer_internal_token) {
        return (StatusCode::UNAUTHORIZED, "forbidden").into_response();
    }

    match state.storage.load_replay(score_id).await {
        Ok(replay) if !replay.is_empty() => Bytes::from(replay).into_response(),
        Ok(_) => (StatusCode::NOT_FOUND, "no replay stored").into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "no replay stored").into_response(),
    }
}
