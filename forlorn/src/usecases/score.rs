use anyhow::Result;
use base64::prelude::*;
use simple_rijndael::{Errors, impls::RijndaelCbc, paddings::Pkcs7Padding};
use webhook::{Author, Embed, Footer, Thumbnail, Webhook};

use crate::{
    config::OmajinaiConfig,
    constants::SubmissionStatus,
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

pub fn first_place_webhook(
    user: &User,
    score: &Score,
    beatmap: &Beatmap,
    webhook_url: &str,
    prev_holder: Option<(i32, String)>,
) -> Webhook {
    let desc = format!(
        "{} ▸ {}pp ({}pp) ▸ {}\n{:.2}% ▸ [{}/{}/{}/{}x] ▸ {}/{}x ▸ {}",
        score.grade().discord_emoji(),
        fmt_f(score.pp),              // formatted to match python
        fmt_f(score.hypothetical_pp), // formatted to match python
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
