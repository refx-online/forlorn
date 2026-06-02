use std::sync::LazyLock;

use anyhow::Result;

use crate::{config::OmajinaiConfig, infrastructure::database::DbPoolManager, models::Beatmap};

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub async fn ensure_osu_file(config: &OmajinaiConfig, beatmap: &Beatmap) -> Result<bool> {
    let url = format!(
        "{}/v1/ensure-osu/{}?md5={}",
        config.beatmap_service_url, beatmap.id, beatmap.md5
    );
    let resp = CLIENT.get(&url).send().await?;
    Ok(resp.status().is_success())
}

pub async fn increment_playcount(
    db: &DbPoolManager,
    beatmap: &mut Beatmap,
    passed: bool,
) -> Result<()> {
    beatmap.plays += 1;
    if passed {
        beatmap.passes += 1;
    }

    sqlx::query("update maps set plays = ?, passes = ? where md5 = ?")
        .bind(beatmap.plays)
        .bind(beatmap.passes)
        .bind(&beatmap.md5)
        .execute(db.as_ref())
        .await?;

    Ok(())
}
