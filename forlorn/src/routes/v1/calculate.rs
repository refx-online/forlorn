use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde_json::{Value, json};

use crate::{
    dto::v1::calculate::GetCalculateMap,
    repository,
    state::AppState,
    usecases::{beatmap::ensure_local_osu_file, score::calculate_performance},
};

const COMMON_ACCURACY: [f32; 5] = [100.0, 99.0, 98.0, 95.0, 90.0];

pub async fn get_calculate_map(
    State(state): State<AppState>,
    Query(calculate): Query<GetCalculateMap>,
) -> (StatusCode, Json<Value>) {
    let user = match repository::user::fetch_by_api_key(&state.db, &calculate.api_key).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!(
                {
                    "reason": "Your api key is revoked/doesn't exists.",
                })),
            );
        },
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(
                {
                    "reason": "Something wen't wrong!",
                    "trace": e.to_string()
                })),
            );
        },
    };

    if user.restricted() {
        return (
            StatusCode::METHOD_NOT_ALLOWED,
            Json(json!(
            {
                "reason": "You're restricted!",
            })),
        );
    }

    let accuracy = calculate.accuracy.unwrap_or(100.0);
    let mode = calculate.mode.unwrap_or(0);
    let mods = calculate.mods.unwrap_or(0);
    let max_combo = calculate.max_combo.unwrap_or(0);
    let legacy_score = calculate.legacy_score.unwrap_or(0);
    let miss_count = calculate.misses.unwrap_or(0);

    let beatmap =
        match repository::beatmap::fetch_by_id(&state.config, &state.db, &calculate.map_id).await {
            Ok(Some(beatmap)) => beatmap,
            _ => {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!(
                    {
                        "reason": "Beatmap doesn't exists.",
                    })),
                );
            },
        };

    let beatmap_info = json!({
        "id": beatmap.id,
        "set_id": beatmap.set_id,
        "status": beatmap.status,
        "md5": beatmap.md5,
        "artist": beatmap.artist,
        "title": beatmap.title,
        "version": beatmap.version,
        "creator": beatmap.creator,
        "total_length": beatmap.total_length,
        "max_combo": beatmap.max_combo,
        "plays": beatmap.plays,
        "passes": beatmap.passes,
        "mode": beatmap.mode,
        "bpm": beatmap.bpm,
        "cs": beatmap.cs,
        "ar": beatmap.ar,
        "od": beatmap.od,
        "hp": beatmap.hp,
        "diff": beatmap.diff,
        "last_update": beatmap.last_update,
    });

    if ensure_local_osu_file(
        state.storage.beatmap_file(beatmap.id),
        &state.config.omajinai,
        &beatmap,
    )
    .await
    .is_err()
    {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "reason": "Beatmap doesn't exists.",
            })),
        );
    }

    let _ = state.metrics.incr("pp.calculated", ["status:ok"]);

    // XXX: if they didn't put accuracy, we can just use `COMMON_ACCURACY`
    if calculate.accuracy.is_none() {
        let mut pp_results = Vec::new();
        let mut stars = 0.0;

        for accuracy in COMMON_ACCURACY {
            let (pp, star, _) = calculate_performance(
                &state.config.omajinai,
                beatmap.id,
                mode,
                mods,
                max_combo,
                accuracy,
                miss_count,
                legacy_score,
            )
            .await;

            if pp.is_nan() || pp.is_infinite() {
                continue;
            }

            stars = star;
            pp_results.push(pp);
        }

        return (
            StatusCode::OK,
            Json(json!({
                "pp": pp_results,
                "stars": stars,
                "beatmap": beatmap_info,
            })),
        );
    }

    let (pp, stars, hypothetical_pp) = calculate_performance(
        &state.config.omajinai,
        beatmap.id,
        mode,
        mods,
        max_combo,
        accuracy,
        miss_count,
        legacy_score,
    )
    .await;

    if pp.is_nan() || pp.is_infinite() || stars.is_nan() || stars.is_infinite() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!(
            {
                "reason": "pp/stars is NaN/Infinite!",
            })),
        );
    }

    (
        StatusCode::OK,
        Json(json!(
        {
            "pp": pp,
            "hypothetical_pp": hypothetical_pp,
            "stars": stars,
            "beatmap": beatmap_info,
        })),
    )
}
