use crate::models::LeaderboardScore;

pub fn format_score_line(score: &LeaderboardScore, rank: i32) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|1",
        score.id,
        score.name,
        score.pp.round() as i64,
        score.max_combo,
        score.n50,
        score.n100,
        score.n300,
        score.nmiss,
        score.nkatu,
        score.ngeki,
        score.perfect as i32,
        score.mods,
        score.userid,
        rank,
        score.play_time,
    )
}
