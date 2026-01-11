use std::collections::HashMap;

use axum::body::Bytes;
use md5::{Digest, Md5};
use tokio::signal::{
    self,
    unix::{self, SignalKind},
};

use crate::{
    dto::{error::GetError, screenshot::ScreenshotUpload, submission::ScoreSubmission},
    models::{Beatmap, LeaderboardScore, MapleAimAssistValues, PersonalBest, Score, Stats, User},
    repository,
    state::AppState,
    usecases::{achievement::check_and_unlock_achievements, leaderboard::format_score_line},
};

// todo: trait

/// x,xxx.xx
pub fn fmt_n(n: i32) -> String {
    n.to_string()
        .chars()
        .rev()
        .enumerate()
        .fold(String::new(), |mut acc, (i, ch)| {
            if i != 0 && i % 3 == 0 {
                acc.push(',');
            }
            acc.push(ch);
            acc
        })
        .chars()
        .rev()
        .collect()
}

/// xx,xxx,xxx
pub fn fmt_f(f: f32) -> String {
    format!("{f:.2}")
        .split_once('.')
        .map(|(int, frac)| {
            int.parse::<i32>()
                .map(|v| format!("{}.{}", fmt_n(v), frac))
                .unwrap_or_else(|_| format!("{f:.2}"))
        })
        .unwrap_or_else(|| format!("{f:.2}"))
}

pub fn build_submission(
    score_data_b64: Vec<u8>,
    replay_file: Vec<u8>,
    fields: HashMap<String, Bytes>,
) -> Option<ScoreSubmission> {
    let get_string = |key: &str| -> Option<String> {
        fields
            .get(key)
            .map(|b| String::from_utf8_lossy(b).to_string())
    };
    let get_i32 = |key: &str| -> i32 { get_string(key).and_then(|s| s.parse().ok()).unwrap_or(0) };
    let get_f32 =
        |key: &str| -> f32 { get_string(key).and_then(|s| s.parse().ok()).unwrap_or(0.0) };
    let get_i8 = |key: &str| -> i8 { get_string(key).and_then(|s| s.parse().ok()).unwrap_or(0) };

    let maple_values =
        get_string("maple").and_then(|s| serde_json::from_str::<MapleAimAssistValues>(&s).ok());

    Some(ScoreSubmission {
        exited_out: get_string("x"),
        fail_time: get_i32("ft"),
        visual_settings_b64: get_string("fs"),
        updated_beatmap_hash: get_string("bmk"),
        storyboard_md5: get_string("sbk"),
        iv_b64: fields.get("iv").cloned()?,
        unique_ids: get_string("c1"),
        score_time: get_i32("st"),
        password_md5: get_string("pass")?,
        osu_version: get_string("osuver")?,
        auth_hash: get_string("cl"),
        client_hash_b64: fields.get("s").cloned()?,
        aim_value: get_i32("acval"),
        ar_value: get_f32("arval"),
        aim_assist_type: get_i8("aatype"),
        arc: get_string("ar"),
        hdr: get_string("hdrem"),
        cs: get_string("cs"),
        tw: get_string("tw"),
        twval: get_f32("twval"),
        refx: get_string("refx"),
        maple_values,
        score_data_b64,
        replay_file,
    })
}

pub fn build_screenshot_upload(fields: HashMap<String, Bytes>) -> Option<ScreenshotUpload> {
    let get_string = |key: &str| -> Option<String> {
        fields
            .get(key)
            .map(|b| String::from_utf8_lossy(b).to_string())
    };
    let get_i32 = |key: &str| -> Option<i32> { get_string(key).and_then(|s| s.parse().ok()) };

    Some(ScreenshotUpload {
        username: get_string("u")?,
        password_md5: get_string("p")?,
        version: get_i32("v"),
        screenshot_data: fields.get("ss")?.to_vec(),
    })
}

pub fn build_error_upload(fields: HashMap<String, Bytes>) -> Option<GetError> {
    let get_string = |key: &str| -> Option<String> {
        fields
            .get(key)
            .map(|b| String::from_utf8_lossy(b).to_string())
    };

    let get_i32 = |key: &str| -> Option<i32> { get_string(key).and_then(|s| s.parse().ok()) };

    Some(GetError {
        username: get_string("u"),
        password_md5: get_string("h"),
        user_id: get_i32("i"),
        stacktrace: get_string("stacktrace"),
        exception: get_string("exception"),
        feedback: get_string("feedback"),
        config: get_string("config")?,
        exe_hash: get_string("exehash")?,
        version: get_string("version")?,
        screenshot_data: fields
            .get("ss")
            .and_then(|b| if b.is_empty() { None } else { Some(b.to_vec()) }),
    })
}

fn chart_entry(key: &str, before: impl std::fmt::Display, after: impl std::fmt::Display) -> String {
    format!("{key}Before:{before}|{key}After:{after}")
}

pub async fn build_submission_charts(
    score: &Score,
    beatmap: &Beatmap,
    stats: &Stats,
    prev_stats: &Stats,
    state: &AppState,
) -> String {
    let mut charts = Vec::new();

    let achievements_str = check_and_unlock_achievements(&state.db, score)
        .await
        .unwrap_or_default();

    charts.push(format!("beatmapId:{}", beatmap.id));
    charts.push(format!("beatmapSetId:{}", beatmap.set_id));
    charts.push(format!("beatmapPlaycount:{}", beatmap.plays));
    charts.push(format!("beatmapPasscount:{}", beatmap.passes));
    charts.push(format!("approvedDate:{}", beatmap.last_update));
    charts.push("\n".to_string());

    charts.push("chartId:beatmap".to_string());
    charts.push(format!(
        "chartUrl:https://refx.online/beatmaps/{}",
        beatmap.set_id
    ));
    charts.push("chartName:Beatmap Ranking".to_string());

    if let Ok(Some(prev_best)) =
        repository::score::fetch_best(&state.db, score.userid, &beatmap.md5, score.mode).await
    {
        charts.push(chart_entry("rank", prev_best.rank, score.rank));
        charts.push(chart_entry("rankedScore", prev_best.score, score.score));
        charts.push(chart_entry("totalScore", prev_best.score, score.score));
        charts.push(chart_entry(
            "maxCombo",
            prev_best.max_combo,
            score.max_combo,
        ));
        charts.push(chart_entry("accuracy", prev_best.acc, score.acc));
        charts.push(chart_entry("pp", prev_best.pp.round(), score.pp.round()));
    } else {
        charts.push(chart_entry("rank", 0, score.rank));
        charts.push(chart_entry("rankedScore", 0, score.score));
        charts.push(chart_entry("totalScore", 0, score.score));
        charts.push(chart_entry("maxCombo", 0, score.max_combo));
        charts.push(chart_entry("accuracy", 0.0, score.acc));
        charts.push(chart_entry("pp", 0.0, score.pp.round()));
    }

    charts.push(format!("onlineScoreId:{}", score.id));
    charts.push("\n".to_string());

    charts.push("chartId:overall".to_string());
    charts.push(format!("chartUrl:https://refx.online/u/{}", score.userid));
    charts.push("chartName:Overall Ranking".to_string());
    charts.push(chart_entry("rank", prev_stats.rank, stats.rank));
    charts.push(chart_entry("rankedScore", prev_stats.rscore, stats.rscore));
    charts.push(chart_entry("totalScore", prev_stats.tscore, stats.tscore));
    charts.push(chart_entry(
        "maxCombo",
        prev_stats.max_combo,
        stats.max_combo,
    ));
    charts.push(chart_entry("accuracy", prev_stats.acc, stats.acc));
    charts.push(chart_entry("pp", prev_stats.pp, stats.pp));
    charts.push(format!("achievements-new:{achievements_str}"));

    charts.join("|")
}

pub fn build_leaderboard_response(
    beatmap: &Beatmap,
    scores: &[LeaderboardScore],
    personal_best: Option<PersonalBest>,
    avg_rating: f32,
    is_refx: bool,
) -> String {
    let mut lines = vec![
        format!(
            "{}|false|{}|{}|{}|0|",
            beatmap.status,
            beatmap.id,
            beatmap.set_id,
            scores.len()
        ),
        format!("0\n{}\n{:.1}", beatmap.full_name(), avg_rating),
    ];

    if let Some(pb) = personal_best {
        lines.push(format_score_line(&pb.score, pb.rank, is_refx));
    } else {
        lines.push(String::new());
    }

    for (idx, score) in scores.iter().enumerate() {
        lines.push(format_score_line(score, (idx + 1) as i32, is_refx));
    }

    lines.join("\n")
}

pub async fn build_empty_leaderboard(beatmap: &Beatmap, state: &AppState) -> String {
    let avg_rating = repository::rating::fetch_average_rating(&state.db, &beatmap.md5)
        .await
        .unwrap_or(0.0);

    let resp = format!(
        "{}|false|{}|{}|0|0|\n0\n{}\n{:.1}\n\n",
        beatmap.status,
        beatmap.id,
        beatmap.set_id,
        beatmap.full_name(),
        avg_rating
    );

    resp
}

pub async fn save_screenshot(state: &AppState, screenshot_data: Vec<u8>) -> String {
    let ext = if screenshot_data.len() > 10
        && (&screenshot_data[6..10] == b"JFIF" || &screenshot_data[6..10] == b"Exif")
    {
        "jpeg"
    } else {
        "png"
    };

    let mut hasher = Md5::new();
    hasher.update(&screenshot_data);

    let hash = format!("{:x}", hasher.finalize());

    let file_name = format!("{}.{}", &hash[..8], ext);

    let _ = state
        .storage
        .save_screenshot(&file_name, &screenshot_data)
        .await;

    file_name
}

pub async fn build_display_name(user: &User, state: &AppState) -> String {
    if let Ok(Some(clan)) = repository::clan::fetch_by_id(&state.db, user.clan_id).await {
        return format!("[{}] {}", clan.tag, user.name);
    }

    user.name.clone()
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        unix::signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("received termination signal, shutting down...");
}
