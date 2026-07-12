use std::collections::HashMap;

use axum::body::Bytes;
use md5::{Digest, Md5};
use tokio::signal::{
    self,
    unix::{self, SignalKind},
};

use serde::Serialize;

use crate::{
    dto::{error::GetError, screenshot::ScreenshotUpload, submission::ScoreSubmission},
    models::{Beatmap, LeaderboardScore, MapleAimAssistValues, PersonalBest, Score, Stats, User},
    repository,
    state::AppState,
    usecases::{achievement::check_and_unlock_achievements, leaderboard::format_score_line},
    constants::{Mods, GameMode},
};

const DATETIME_OFFSET: i64 = 621_355_968_000_000_000;

pub fn generate_lazer_info(
    score_id: u64,
    mode: i32,
    mods: i32,
    clock_rate: f64,
    n300: i32,
    n100: i32,
    n50: i32,
    ngeki: i32,
    nkatu: i32,
    nmiss: i32,
) -> Vec<u8> {
    #[derive(Serialize)]
    struct ApiMod {
        acronym: String,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        settings: HashMap<String, serde_json::Value>,
    }

    #[derive(Serialize)]
    struct ScoreInfo {
        online_id: i64,
        mods: Vec<ApiMod>,
        statistics: HashMap<String, i64>,
        maximum_statistics: HashMap<String, i64>,
    }

    let mut mod_list = Vec::new();

    if mode >= 12 {
        mod_list.push(ApiMod {
            acronym: "CL".into(),
            settings: HashMap::new(),
        });
    }

    let mod_bits = Mods::from_bits_truncate(mods);
    let speed = if clock_rate > 0.0 { clock_rate } else { 1.0 };

    const MOD_ENTRIES: &[(Mods, &str)] = &[
        (Mods::NOFAIL, "NF"), (Mods::EASY, "EZ"), (Mods::TOUCHSCREEN, "TD"),
        (Mods::HIDDEN, "HD"), (Mods::HARDROCK, "HR"), (Mods::SUDDENDEATH, "SD"),
        (Mods::DOUBLETIME, "DT"), (Mods::RELAX, "RX"), (Mods::HALFTIME, "HT"),
        (Mods::NIGHTCORE, "NC"), (Mods::FLASHLIGHT, "FL"), (Mods::AUTOPLAY, "AT"),
        (Mods::SPUNOUT, "SO"), (Mods::AUTOPILOT, "AP"), (Mods::PERFECT, "PF"),
        (Mods::KEY4, "K4"), (Mods::KEY5, "K5"), (Mods::KEY6, "K6"),
        (Mods::KEY7, "K7"), (Mods::KEY8, "K8"), (Mods::FADEIN, "FI"),
        (Mods::RANDOM, "RD"), (Mods::CINEMA, "CN"), (Mods::TARGET, "TP"),
        (Mods::KEY9, "K9"), (Mods::KEYCOOP, "KC"), (Mods::KEY1, "K1"),
        (Mods::KEY3, "K3"), (Mods::KEY2, "K2"), (Mods::SCOREV2, "V2"),
        (Mods::MIRROR, "MR"),
    ];

    let skip_dt = mod_bits.contains(Mods::NIGHTCORE);

    for &(flag, acronym) in MOD_ENTRIES {
        if skip_dt && flag == Mods::DOUBLETIME {
            continue;
        }
        if !mod_bits.contains(flag) {
            continue;
        }

        let mut settings = HashMap::new();
        if matches!(flag, Mods::DOUBLETIME | Mods::NIGHTCORE | Mods::HALFTIME) {
            settings.insert("speed_change".into(), serde_json::Value::from(speed));
        }
        mod_list.push(ApiMod { acronym: acronym.into(), settings });
    }

    let vanilla_mode = GameMode::from_params(mode, mod_bits).as_vanilla();
    let mut statistics = HashMap::new();
    let mut maximum_statistics = HashMap::new();

    match vanilla_mode {
        0 => {
            statistics.insert("great".into(), n300 as i64);
            statistics.insert("ok".into(), n100 as i64);
            statistics.insert("meh".into(), n50 as i64);
            statistics.insert("miss".into(), nmiss as i64);
            let total = n300 + n100 + n50 + nmiss;
            maximum_statistics.insert("great".into(), total as i64);
        },
        1 => {
            statistics.insert("great".into(), n300 as i64);
            statistics.insert("ok".into(), n100 as i64);
            statistics.insert("miss".into(), nmiss as i64);
            let total = n300 + n100 + nmiss;
            maximum_statistics.insert("great".into(), total as i64);
        },
        2 => {
            statistics.insert("great".into(), n300 as i64);
            statistics.insert("large_tick_hit".into(), n100 as i64);
            statistics.insert("small_tick_hit".into(), n50 as i64);
            statistics.insert("small_tick_miss".into(), nkatu as i64);
            statistics.insert("miss".into(), nmiss as i64);
            let total = n300 + n100 + n50 + nkatu + nmiss;
            maximum_statistics.insert("great".into(), total as i64);
        },
        3 => {
            statistics.insert("perfect".into(), ngeki as i64);
            statistics.insert("great".into(), n300 as i64);
            statistics.insert("good".into(), nkatu as i64);
            statistics.insert("ok".into(), n100 as i64);
            statistics.insert("meh".into(), n50 as i64);
            statistics.insert("miss".into(), nmiss as i64);
            let total = ngeki + n300 + nkatu + n100 + n50 + nmiss;
            maximum_statistics.insert("perfect".into(), total as i64);
        },
        _ => {}
    }

    let info = ScoreInfo {
        online_id: score_id as i64,
        mods: mod_list,
        statistics,
        maximum_statistics,
    };

    let json = serde_json::to_vec(&info).expect("lazer info serialization");
    let mut compressed = Vec::new();
    lzma_rs::lzma_compress(&mut &json[..], &mut compressed).expect("lzma compression");
    compressed
}

pub fn write_osu_string(buf: &mut Vec<u8>, s: &str) {
    if s.is_empty() {
        buf.push(0x00);
        return;
    }
    buf.push(0x0b);
    let len = s.len();
    if len < 0x80 {
        buf.push(len as u8);
    } else {
        let mut remaining = len;
        loop {
            let mut byte = (remaining & 0x7f) as u8;
            remaining >>= 7;
            if remaining > 0 {
                byte |= 0x80;
            }
            buf.push(byte);
            if remaining == 0 {
                break;
            }
        }
    }
    buf.extend_from_slice(s.as_bytes());
}

pub fn build_osr_replay(
    raw_replay: &[u8],
    mode: i32,
    n300: i32,
    n100: i32,
    n50: i32,
    ngeki: i32,
    nkatu: i32,
    nmiss: i32,
    score: i32,
    max_combo: i32,
    perfect: bool,
    mods: i32,
    play_time: chrono::DateTime<chrono::Utc>,
    score_id: u64,
    map_md5: &str,
    username: &str,
    lazer_data: Option<&[u8]>,
) -> Vec<u8> {
    let lazer_len = lazer_data.map_or(0, |d| d.len());
    let mut buf = Vec::with_capacity(256 + raw_replay.len() + lazer_len);

    // game mode (vanilla)
    let gm = GameMode::from_params(mode, Mods::from_bits_truncate(mods));
    buf.push(gm.as_vanilla() as u8);
    // osu! client version
    // Version >= 30000001 signals additional_info block after online_score_id
    if lazer_data.is_some() {
        buf.extend_from_slice(&30000001i32.to_le_bytes());
    } else {
        buf.extend_from_slice(&20200207i32.to_le_bytes());
    }
    // beatmap md5
    write_osu_string(&mut buf, map_md5);
    // player name
    write_osu_string(&mut buf, username);
    // replay md5
    write_osu_string(&mut buf, &compute_replay_md5_raw(n300, n100, n50, ngeki, nkatu, nmiss, score, max_combo, perfect, mods, map_md5, username));
    // hit counts (short / i16)
    buf.extend_from_slice(&(n300 as i16).to_le_bytes());
    buf.extend_from_slice(&(n100 as i16).to_le_bytes());
    buf.extend_from_slice(&(n50 as i16).to_le_bytes());
    buf.extend_from_slice(&(ngeki as i16).to_le_bytes());
    buf.extend_from_slice(&(nkatu as i16).to_le_bytes());
    buf.extend_from_slice(&(nmiss as i16).to_le_bytes());
    // total score
    buf.extend_from_slice(&score.to_le_bytes());
    // max combo
    buf.extend_from_slice(&(max_combo as i16).to_le_bytes());
    // perfect
    buf.push(if perfect { 1u8 } else { 0u8 });
    // mods
    buf.extend_from_slice(&mods.to_le_bytes());
    // life bar graph (empty string ⇒ 0x00)
    buf.push(0x00);
    // play date (Windows NT ticks)
    let ticks = (play_time.timestamp() as i64 * 10_000_000) + DATETIME_OFFSET;
    buf.extend_from_slice(&ticks.to_le_bytes());
    // replay data length + data
    buf.extend_from_slice(&(raw_replay.len() as i32).to_le_bytes());
    buf.extend_from_slice(raw_replay);
    // online score id
    buf.extend_from_slice(&(score_id as i64).to_le_bytes());
    // additional info
    if let Some(lzr) = lazer_data {
        buf.extend_from_slice(&(lzr.len() as i32).to_le_bytes());
        buf.extend_from_slice(lzr);
    }

    buf
}

fn compute_replay_md5_raw(
    n300: i32, n100: i32, n50: i32, ngeki: i32, nkatu: i32, nmiss: i32,
    score: i32, max_combo: i32, perfect: bool, mods: i32,
    map_md5: &str, username: &str,
) -> String {
    let input = format!(
        "{}p{}o{}o{}t{}a{}r{}e{}y{}o{}u{}{}{}",
        n100 + n300,
        n50,
        ngeki,
        nkatu,
        nmiss,
        map_md5,
        max_combo,
        if perfect { "True" } else { "False" },
        username,
        score,
        0,
        mods,
        "True",
    );
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

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
    lazer_data: Vec<u8>,
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
    let get_f64 =
        |key: &str| -> f64 { get_string(key).and_then(|s| s.parse().ok()).unwrap_or(0.0) };
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
        clock_rate: get_f64("clock_rate_just_in_case"),
        maple_values,
        score_data_b64,
        lazer_data,
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
