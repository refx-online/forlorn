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
            score.aim_correction_value,
            score.ar_changer_value,
            score.uses_aim_correction as i32,
            score.uses_ar_changer as i32,
            score.uses_cs_changer as i32,
            score.uses_timewarp as i32,
            score.timewarp_value,
            score.uses_hd_remover as i32,
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
