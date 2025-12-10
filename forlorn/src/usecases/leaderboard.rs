use crate::models::LeaderboardScore;

pub fn format_score_line(score: &LeaderboardScore, rank: i32, is_refx: bool) -> String {
    if is_refx {
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|1|{}|{}|{}|{}|{}|{}|{}|{}",
            score.id,
            score.name,
            score.preferred_metric.round() as i64,
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
            score.aim_correction_value.unwrap_or(0),
            score.ar_changer_value.unwrap_or(0.0),
            score.uses_aim_correction.unwrap_or(false) as i32,
            score.uses_ar_changer.unwrap_or(false) as i32,
            score.uses_cs_changer.unwrap_or(false) as i32,
            score.uses_timewarp.unwrap_or(false) as i32,
            score.timewarp_value.unwrap_or(0.0),
            score.uses_hd_remover.unwrap_or(false) as i32,
        )
    } else {
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|1",
            score.id,
            score.name,
            score.preferred_metric.round() as i64,
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
}
