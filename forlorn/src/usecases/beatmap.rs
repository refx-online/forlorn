use std::fs;

use anyhow::Result;
use md5::{Digest, Md5};
use storage::Storage;

use crate::{
    config::OmajinaiConfig,
    infrastructure::{database::DbPoolManager, omajinai::beatmap::fetch_beatmap},
    models::Beatmap,
};

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

pub async fn ensure_local_osu_file(
    storage: &Storage,
    config: &OmajinaiConfig,
    beatmap: &Beatmap,
) -> Result<bool> {
    let osu_file_path = storage.beatmap_file(beatmap.id);
    let osu_file_bytes = if osu_file_path.exists() {
        fs::read(&osu_file_path)?
    } else {
        match storage.load_beatmap(beatmap.id).await {
            Ok(bytes) if !bytes.is_empty() => {
                let _ = storage.save_beatmap(beatmap.id, &bytes).await;

                bytes
            },
            _ => {
                tracing::info!(
                    "fetching <{} ({})> from beatmap service.",
                    beatmap.full_name(),
                    beatmap.id
                );
                let bytes = fetch_beatmap(config, beatmap.id).await?;
                let _ = storage.save_beatmap(beatmap.id, &bytes).await;

                bytes
            },
        }
    };
    let mut md5_hasher = Md5::new();
    md5_hasher.update(&osu_file_bytes);

    let osu_file_md5 = format!("{:x}", md5_hasher.finalize());

    Ok(osu_file_md5 == beatmap.md5)
}
