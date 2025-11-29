use crate::models::{Beatmap, Score, Stats};
use crate::repository;
use crate::state::AppState;

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

    let achievements_str = if beatmap.awards_ranked_pp() {
        // TODO: implement achievement
        String::new()
    } else {
        String::new()
    };

    charts.push(format!("beatmapId:{}", beatmap.id));
    charts.push(format!("beatmapSetId:{}", beatmap.set_id));
    charts.push(format!("beatmapPlaycount:{}", beatmap.plays));
    charts.push(format!("beatmapPasscount:{}", beatmap.passes));
    charts.push(format!("approvedDate:{}", beatmap.last_update));
    charts.push("\n".to_string());

    charts.push("chartId:beatmap".to_string());
    charts.push(format!(
        "chartUrl:https://remeliah.cyou/beatmaps/{}",
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
        charts.push(chart_entry(
            "accuracy",
            format!("{:.2}", prev_best.acc),
            format!("{:.2}", score.acc),
        ));
        charts.push(chart_entry(
            "pp",
            format!("{:.2}", prev_best.pp),
            format!("{:.2}", score.pp),
        ));
    } else {
        charts.push(chart_entry("rank", 0, score.rank));
        charts.push(chart_entry("rankedScore", 0, score.score));
        charts.push(chart_entry("totalScore", 0, score.score));
        charts.push(chart_entry("maxCombo", 0, score.max_combo));
        charts.push(chart_entry("accuracy", "0.00", format!("{:.2}", score.acc)));
        charts.push(chart_entry("pp", "0.00", format!("{:.2}", score.pp)));
    }

    charts.push(format!("onlineScoreId:{}", score.id));
    charts.push("\n".to_string());

    charts.push("chartId:overall".to_string());
    charts.push(format!("chartUrl:https://remeliah.cyou/u/{}", score.userid));
    charts.push("chartName:Overall Ranking".to_string());
    charts.push(chart_entry("rank", prev_stats.rank, stats.rank));
    charts.push(chart_entry("rankedScore", prev_stats.rscore, stats.rscore));
    charts.push(chart_entry("totalScore", prev_stats.tscore, stats.tscore));
    charts.push(chart_entry(
        "maxCombo",
        prev_stats.max_combo,
        stats.max_combo,
    ));
    charts.push(chart_entry(
        "accuracy",
        format!("{:.2}", prev_stats.acc * 100.0),
        format!("{:.2}", stats.acc * 100.0),
    ));
    charts.push(chart_entry("pp", prev_stats.pp, stats.pp));
    charts.push(format!("achievements-new:{achievements_str}"));

    charts.join("|")
}

#[allow(unused)] // will be used later
fn format_achievement_string(file: &str, name: &str, desc: &str) -> String {
    format!("{file}+{name}+{desc}")
}
