use axum::{
    extract::{Form, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{repository, state::AppState, usecases::password::verify_password};

#[derive(Debug, Deserialize, Default)]
pub struct PerformanceReport {
    #[serde(default)]
    pub us: String,
    #[serde(default)]
    pub ha: String,
    #[serde(default)]
    pub scoreid: Option<i64>,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub os: String,
    #[serde(default)]
    pub fullscreen: i32,
    #[serde(default)]
    pub fps_cap: String,
    #[serde(default)]
    pub compatibility: i32,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub start_time: i64,
    #[serde(default)]
    pub end_time: i64,
    #[serde(default)]
    pub frame_count: i64,
    #[serde(default)]
    pub spike_frames: i64,
    #[serde(default)]
    pub aim_rate: i64,
    #[serde(default)]
    pub completion: i32,
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub average_frametime: i64,
}

// NOTE: the mod_mode enum only ever had vanilla/relax/autopilot — the
// cheat variants below match the ALTER in init.sql.
fn mod_mode(mode: i32) -> &'static str {
    match mode {
        4..=6 => "relax",
        8 => "autopilot",
        12..=15 => "cheat",
        21..=23 => "cheat-rx",
        24 => "cheat-ap",
        _ => "vanilla",
    }
}

pub async fn submit_performance(
    State(state): State<AppState>,
    Form(report): Form<PerformanceReport>,
) -> impl IntoResponse {
    let user = match repository::user::fetch_by_name(&state.db, &report.us).await {
        Ok(Some(user)) => user,
        _ => return (StatusCode::OK, "error: nouser").into_response(),
    };
    if !verify_password(&report.ha, &user.pw_bcrypt)
        .await
        .unwrap_or(false)
    {
        return (StatusCode::OK, "error: pass").into_response();
    }

    let score = if let Some(id) = report.scoreid {
        match repository::score::fetch_by_id(&state.db, id as u64).await {
            Ok(Some(score)) => score,
            _ => return (StatusCode::OK, "error: no score").into_response(),
        }
    } else if let Some(checksum) = report.checksum.as_deref() {
        match repository::score::fetch_by_online_checksum(&state.db, checksum).await {
            Ok(Some(score)) => score,
            _ => return (StatusCode::OK, "error: no score").into_response(),
        }
    } else {
        return (StatusCode::OK, "error: no score").into_response();
    };

    if score.userid != user.id {
        return (StatusCode::OK, "error: no").into_response();
    }

    let mode = mod_mode(score.mode);
    let result = sqlx::query(
        "INSERT INTO performance_reports
            (scoreid, mod_mode, os, fullscreen, fps_cap, compatibility, `version`,
             start_time, end_time, frame_count, spike_frames, aim_rate,
             completion, identifier, average_frametime)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE
            os = VALUES(os), fullscreen = VALUES(fullscreen),
            fps_cap = VALUES(fps_cap), compatibility = VALUES(compatibility),
            `version` = VALUES(`version`), start_time = VALUES(start_time),
            end_time = VALUES(end_time), frame_count = VALUES(frame_count),
            spike_frames = VALUES(spike_frames), aim_rate = VALUES(aim_rate),
            completion = VALUES(completion), identifier = VALUES(identifier),
            average_frametime = VALUES(average_frametime)",
    )
    .bind(score.id as i64)
    .bind(mode)
    .bind(&report.os)
    .bind(report.fullscreen)
    .bind(&report.fps_cap)
    .bind(report.compatibility)
    .bind(&report.version)
    .bind(report.start_time as i32)
    .bind(report.end_time as i32)
    .bind(report.frame_count as i32)
    .bind(report.spike_frames as i32)
    .bind(report.aim_rate as i32)
    .bind(report.completion != 0)
    .bind(report.identifier.as_deref().unwrap_or(""))
    .bind(report.average_frametime as i32)
    .execute(state.db.as_ref())
    .await;

    if let Err(e) = result {
        tracing::warn!("performance report insert failed: {e:?}");
        return (StatusCode::OK, "error: no").into_response();
    }

    let _ = state
        .metrics
        .incr("performance.reported", ["status:ok"]);

    (StatusCode::OK, "ok").into_response()
}
