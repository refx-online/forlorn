use base64::prelude::*;
use simple_rijndael::impls::RijndaelCbc;
use simple_rijndael::paddings::Pkcs7Padding;

use crate::constants::SubmissionStatus;
use crate::models::Score;
use anyhow::Result;

pub fn decrypt_score_data(
    score_data_b64: &[u8],
    client_hash_b64: &[u8],
    iv_b64: &[u8],
    osu_version: &str,
) -> Result<(Vec<String>, String), simple_rijndael::Errors> {
    let aes = RijndaelCbc::<Pkcs7Padding>::new(
        format!("osu!-scoreburgr---------{}", osu_version).as_bytes(),
        32,
    )?;

    let iv = BASE64_STANDARD.decode(iv_b64).unwrap();

    let score_data: Vec<String> = {
        let b = aes.decrypt(&iv, BASE64_STANDARD.decode(score_data_b64).unwrap())?;

        String::from_utf8_lossy(&b)
            .split(':')
            .map(|s| s.to_string())
            .collect()
    };

    let client_hash_decoded: String = {
        let b = aes.decrypt(&iv, BASE64_STANDARD.decode(client_hash_b64).unwrap())?;

        String::from_utf8_lossy(&b).to_string()
    };

    Ok((score_data, client_hash_decoded))
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

pub fn calculate_status(new_score: &mut Score, previous_best: Option<Score>) -> Option<Score> {
    match previous_best {
        Some(mut prev_best) => {
            // we have a score on the map.
            // if our new score is better, update both submission statuses.
            if new_score.pp > prev_best.pp {
                new_score.status = SubmissionStatus::BEST.as_i32();
                prev_best.status = SubmissionStatus::SUBMITTED.as_i32();
            } else {
                new_score.status = SubmissionStatus::SUBMITTED.as_i32();
            }
            Some(prev_best)
        },
        None => {
            // this is our first score on the map.
            new_score.status = SubmissionStatus::BEST.as_i32();
            None
        },
    }
}
