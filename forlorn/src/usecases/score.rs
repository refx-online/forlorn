use anyhow::Result;
use base64::prelude::*;
use simple_rijndael::{Errors, impls::RijndaelCbc, paddings::Pkcs7Padding};
use webhook::{Author, Embed, Footer, Thumbnail, Webhook};

use crate::{
    config::OmajinaiConfig,
    constants::{GameMode, Grade, Mods, SubmissionStatus},
    dto::submission::ScoreSubmission,
    infrastructure::{
        database::DbPoolManager,
        omajinai::{PerformanceRequest, calculate_pp},
    },
    models::{Beatmap, Score, User},
    repository,
    utils::{fmt_f, fmt_n},
};

const DISCORD_SCORE_EMBED_COLOR: u32 = 0x2B2D31;

pub fn decrypt_score_data(
    score_data_b64: &[u8],
    client_hash_b64: &[u8],
    iv_b64: &[u8],
    osu_version: &str,
) -> Result<(Vec<String>, String), Errors> {
    let aes = RijndaelCbc::<Pkcs7Padding>::new(
        format!("osu!-scoreburgr---------{osu_version}").as_bytes(),
        32,
    )?;

    let iv = BASE64_STANDARD
        .decode(iv_b64)
        .map_err(|_| Errors::InvalidDataSize)?;

    let score_data: Vec<String> = {
        let b = aes.decrypt(
            &iv,
            BASE64_STANDARD
                .decode(score_data_b64)
                .map_err(|_| Errors::InvalidDataSize)?,
        )?;

        String::from_utf8_lossy(&b)
            .split(':')
            .map(|s| s.to_string())
            .collect()
    };

    let client_hash_decoded: String = {
        let b = aes.decrypt(
            &iv,
            BASE64_STANDARD
                .decode(client_hash_b64)
                .map_err(|_| Errors::InvalidDataSize)?,
        )?;

        String::from_utf8_lossy(&b).to_string()
    };

    let osu_path_md5 = client_hash_decoded
        .split_once(':')
        .map_or(client_hash_decoded.clone(), |(full_path, _)| {
            full_path.to_string()
        });

    Ok((score_data, osu_path_md5))
}

pub fn calculate_accuracy(score: &Score) -> f32 {
    let mode = score.mode().as_vanilla();

    let n300 = score.n300 as f32;
    let n100 = score.n100 as f32;
    let n50 = score.n50 as f32;

    let ngeki = score.ngeki as f32;
    let nkatu = score.nkatu as f32;

    let nmiss = score.nmiss as f32;

    match mode {
        0 => {
            let total = n300 + n100 + n50 + nmiss;
            if total == 0.0 {
                return 0.0;
            }
            100.0 * ((n300 * 300.0) + (n100 * 100.0) + (n50 * 50.0)) / (total * 300.0)
        },
        1 => {
            let total = n300 + n100 + nmiss;
            if total == 0.0 {
                return 0.0;
            }
            100.0 * ((n100 * 0.5) + n300) / total
        },
        2 => {
            let total = n300 + n100 + n50 + nkatu + nmiss;
            if total == 0.0 {
                return 0.0;
            }
            100.0 * (n300 + n100 + n50) / total
        },
        3 => {
            let total = n300 + n100 + n50 + ngeki + nkatu + nmiss;
            if total == 0.0 {
                return 0.0;
            }
            100.0 * ((n50 * 50.0) + (n100 * 100.0) + (nkatu * 200.0) + ((n300 + ngeki) * 300.0))
                / (total * 300.0)
        },
        _ => 0.0,
    }
}

pub async fn calculate_status(db: &DbPoolManager, new_score: &mut Score) -> Result<Option<Score>> {
    let previous_best =
        repository::score::fetch_best(db, new_score.userid, &new_score.map_md5, new_score.mode)
            .await?;

    match previous_best {
        Some(mut prev_best) => {
            // we have a score on the map.
            // if our new score is better, update both submission statuses.
            if new_score.pp > prev_best.pp {
                new_score.status = SubmissionStatus::Best.as_i32();
                prev_best.status = SubmissionStatus::Submitted.as_i32();

                Ok(Some(prev_best))
            } else {
                new_score.status = SubmissionStatus::Submitted.as_i32();

                Ok(None)
            }
        },
        None => {
            // this is our first score on the map.
            new_score.status = SubmissionStatus::Best.as_i32();

            Ok(None)
        },
    }
}

pub async fn calculate_placement(db: &DbPoolManager, score: &Score) -> u32 {
    let num_better_scores = repository::score::fetch_num_better_scores(db, score).await;

    num_better_scores.unwrap_or_default()
}

pub async fn update_any_preexisting_personal_best(db: &DbPoolManager, score: &Score) -> () {
    let _ = repository::score::update_preexisting_personal_best(db, score).await;
}

#[allow(clippy::too_many_arguments)]
pub async fn calculate_performance(
    config: &OmajinaiConfig,
    beatmap_id: i32,
    mode: i32,
    mods: i32,
    max_combo: i32,
    accuracy: f32,
    miss_count: i32,
    legacy_score: i32,
) -> (f32, f32, f32) {
    let request = PerformanceRequest {
        beatmap_id,
        mode,
        mods,
        max_combo,
        accuracy,
        miss_count,
        legacy_score,
    };

    match calculate_pp(config, &request).await {
        Ok(result) => (result.pp, result.stars, result.hypothetical_pp),
        Err(_) => (0.0, 0.0, 0.0), // TODO: raise for error instead setting to 0? but it will broke submission..
    }
}

/// This xp calculation that was supposed to
/// replace "Performance Point" for the cheat/cheatcheat mode.
/// It was implemented by kaupec1 when the server was a cheat only (early days),
/// and I have to say this calculation is very weird.
///
/// But oh well, I deprecated it. this is here since I just remember
/// that this calculation exists back then. and then I ported it today on
/// Rust since the codebase starts to look better now. and for legacy purposes.
/// and most importantly, I have free time and it needs to be wasted.
///
/// Also I edited some value to match up the current state
/// since this server has like billion leaderboard now.
#[rustfmt::skip]
pub fn calculate_xp(score: &Score, beatmap: &Beatmap) -> f32 {
    let (
        score_weight,
        pp_weight,
        combo_weight,
        time_weight,
        acc_weight,
        ar_weight,
        aim_weight,
        timewarp_weight,
        cs_base_penalty,
        hd_penalty_multiplier,
        perfect_multiplier,
    ) = (
        75.0,
        50.0,
        25.0,
        60.0,
        50.0,
        25.0,
        25.0,
        20.0,
        0.10,
        0.10,
        1.25,
    );

    let grade_multiplier = match score.grade() {
        Grade::XH => 1.70,
        Grade::SH => 1.50,
        Grade::X => 1.35,
        Grade::S => 1.20,
        Grade::A => 1.10,
        Grade::B => 1.05,
        Grade::C => 1.02,
        Grade::D => 1.00,
        _ => 1.00,
    };

    let status_bonus = match score.status() {
        SubmissionStatus::Best => 50.0,
        SubmissionStatus::Submitted => 20.0,
        _ => 0.0,
    };

    let mut xp = 0.0;

    let score_normalized =
        (score.score / i32::MAX).min(1) as f32;

    xp += score_weight 
        * (1.0 - (-22.5 * score_normalized).exp());

    let pp_normalized =
        (score.pp / score.hypothetical_pp).min(1.0);

    xp += pp_normalized * pp_weight;

    let max_combo_normalized =
        (score.max_combo / beatmap.max_combo).min(1) as f32;

    xp += max_combo_normalized * combo_weight;

    let time_elapsed_normalized =
        ((beatmap.total_length as f32).ln_1p() / 10.0).min(1.0);

    xp += time_elapsed_normalized * time_weight;

    let acc_normalized = 1.0
        / (1.0 + (-(score.acc / 100.0 - 0.5)).exp());
    let acc_exponential = 0.5
        * acc_normalized.powf(2.0) + 1.0 * acc_normalized + 20.0;
    let acc_penalty = if score.acc < 85.0 {
        -(2.0 * (85.0 - score.acc) / 75.0).exp()
    } else {
        0.0
    };

    xp += (acc_exponential + acc_penalty) * acc_weight;

    if score.mode().cheat() {
        if score.ar_changer_value > -1.0 {
            let ar_changer_value_normalized = if score.ar_changer_value < 0.0 {
                0.0
            } else if score.ar_changer_value < 6.0 || score.ar_changer_value > 10.0 {
                1.0 - ((score.ar_changer_value - 6.0) / 6.0).abs()
            } else {
                1.0 - ((score.ar_changer_value - 6.1) / (9.9 - 6.1))
            };

            xp += ar_changer_value_normalized * ar_weight;
        }

        if score.aim_correction_value > -1 {
            let aim_correction_limit = if score.mode() == GameMode::CHEAT_OSU {
                60
            } else {
                80
            };
            let aim_correction_value_normalized =
                (score.aim_correction_value / aim_correction_limit).min(1) as f32;

            xp += aim_correction_value_normalized * aim_weight;
        }

        if score.timewarp_value > 0.0 {
            let timewarp_speed = score.timewarp_value / 100.0;

            let expected_speed = if score.mods().contains(Mods::DOUBLETIME) || score.mods().contains(Mods::NIGHTCORE) {
                1.5
            } else if score.mods().contains(Mods::HALFTIME) {
                0.75
            } else {
                1.0
            };

            let timewarp_deviation = timewarp_speed - expected_speed;

            const TIMEWARP_MIN: f32 = 0.9;

            let timewarp_normalized = if timewarp_deviation.abs() < 0.01 {
                0.0
            } else if timewarp_deviation > 0.0 {
                // +0.1 (10% faster) = +0.2, +0.5 (50% faster) = +0.5, +1.0 = +1.0
                (timewarp_deviation * 2.0).clamp(0.0, 1.0)
            } else {
                let actual_slowdown = (expected_speed - timewarp_speed).max(0.0);
                let max_possible_slowdown = (expected_speed - TIMEWARP_MIN).max(0.0);

                if actual_slowdown > 0.0 && max_possible_slowdown > 0.0 {
                    // NM (1.0->0.9): 0.1/0.1 = 100% of possible = -0.5
                    // DT (1.5->0.9): 0.6/0.6 = 100% of possible = -0.5
                    // DT (1.5->1.2): 0.3/0.6 = 50% of possible = -0.25
                    // yes i pulled this formula out of my ass
                    -(actual_slowdown / max_possible_slowdown).min(1.0) * 0.5
                } else {
                    0.0
                }
            };

            xp += timewarp_normalized * timewarp_weight;
        }

        if score.uses_cs_changer {
            let cs_reduction = if score.mods().contains(Mods::HARDROCK) {
                1.0 
            } else {
                // ez/nm
                // im not sure why the cs changer logic that way?
                0.77
            };
            
            // lower CS = bigger circles = easier to hit
            // penalty is larger when reducing already-low circle size
            let cs_difficulty_factor = if beatmap.cs > 0.0 {
                // normalize: circle size 2-7 range, penalty increases as original circle size decreases
                let normalized_cs = ((beatmap.cs - 2.0) / 5.0).clamp(0.0, 1.0);

                // lower original CS = higher penalty multiplier
                let reduction_impact = cs_reduction / beatmap.cs;

                1.0 - (normalized_cs * 0.5 + 0.5) * reduction_impact
            } else {
                1.0 - cs_base_penalty
            };

            let cs_penalty_multiplier = cs_difficulty_factor.clamp(0.80, 0.95);
            xp *= cs_penalty_multiplier;
        }

        if score.uses_hd_remover && score.mods().contains(Mods::HIDDEN) {
            xp *= 1.0 - hd_penalty_multiplier;
        }
    }

    if score.perfect {
        xp *= perfect_multiplier;
    }

    xp *= grade_multiplier;

    xp += status_bonus;

    xp.max(0.0)
}

pub fn consume_cheat_values(score: &mut Score, fields: &ScoreSubmission) {
    score.uses_aim_correction = fields.aim();
    score.aim_correction_value = fields.aim_value;
    score.uses_ar_changer = fields.arc();
    score.ar_changer_value = fields.ar_value;
    score.uses_timewarp = fields.tw();
    score.timewarp_value = fields.twval;
    score.uses_cs_changer = fields.cs();
    score.uses_hd_remover = fields.hdr();
}

pub fn validate_cheat_values(score: &Score) -> bool {
    match score.mode() {
        GameMode::CHEAT_OSU => {
            if score.uses_aim_correction && score.aim_correction_value > 60 {
                return false;
            }
            if score.uses_timewarp || score.timewarp_value != -1.0 {
                return false;
            }
            if score.uses_cs_changer {
                return false;
            }

            true
        },
        GameMode::CHEAT_CHEAT_OSU => {
            if score.uses_aim_correction && score.aim_correction_value > 80 {
                return false;
            }
            if score.uses_timewarp && score.timewarp_value < 90.0 {
                return false;
            }

            true
        },
        _ => true,
    }
}

pub fn first_place_webhook(
    user: &User,
    score: &Score,
    beatmap: &Beatmap,
    webhook_url: &str,
    prev_holder: Option<(i32, String)>,
) -> Webhook {
    let desc = format!(
        "{} ▸ {}pp ({}pp) [{}xp] ▸ {}\n{:.2}% ▸ [{}/{}/{}/{}x] ▸ {}/{}x ▸ {}",
        score.grade().discord_emoji(),
        fmt_f(score.pp),              // formatted to match python
        fmt_f(score.hypothetical_pp), // formatted to match python
        fmt_f(score.xp),              // formatted to match python
        fmt_n(score.score),           // formatted to match python
        score.acc,
        score.n300,
        score.n100,
        score.n50,
        score.nmiss,
        score.max_combo,
        beatmap.max_combo,
        score.mods().as_str()
    );

    #[allow(clippy::uninlined_format_args)]
    let content: String = if let Some((prev_id, prev_name)) = prev_holder {
        format!(
            "previously held by [{}](https://refx.online/u/{})",
            prev_name, prev_id
        )
    } else {
        String::new()
    };

    // TODO: pp record announce

    let embed = Embed::new()
        .title(format!("{} - {:.2}★", beatmap.full_name(), score.stars))
        .url(beatmap.url())
        .description(desc)
        .color(DISCORD_SCORE_EMBED_COLOR) // gray
        .author(Author::new().name(format!("set a new #1 worth {:.2}pp", score.pp)))
        .thumbnail(Thumbnail::new().url(format!(
            "https://assets.ppy.sh/beatmaps/{}/covers/card.jpg",
            beatmap.set_id
        )))
        .footer(Footer::new(format!("{} | forlorn", score.mode().as_str())));

    Webhook::new(webhook_url)
        .username(&user.name)
        .content(content)
        .avatar_url(format!("https://a.refx.online/{}", user.id))
        .add_embed(embed)
}
