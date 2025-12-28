use crate::models::{AimAssistType, LeaderboardScore};

pub fn format_score_line(score: &LeaderboardScore, rank: i32, is_refx: bool) -> String {
    if is_refx {
        let maple_json = if score.aim_assist_type() == AimAssistType::MapleAimAssist {
            score
                .maple_values
                .as_ref()
                .map(|m| serde_json::to_string(&m.0).unwrap_or_default())
                .unwrap_or_default()
        } else {
            String::new()
        };

        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|1|{}|{}|{}|{}|{}|{}|{}|{}|{}",
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
            score.aim_assist_type.unwrap_or(0),
            score.aim_correction_value.unwrap_or(0),
            score.ar_changer_value.unwrap_or(0.0),
            score.uses_ar_changer.unwrap_or(false) as i32,
            score.uses_cs_changer.unwrap_or(false) as i32,
            score.uses_timewarp.unwrap_or(false) as i32,
            score.timewarp_value.unwrap_or(0.0),
            score.uses_hd_remover.unwrap_or(false) as i32,
            maple_json,
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
