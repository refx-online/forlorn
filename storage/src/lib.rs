use std::{fs, path::PathBuf, sync::Arc};

use anyhow::Result;
use cloudflare_r2_rs::r2::R2Manager;

#[derive(Clone)]
pub struct Storage {
    replay_path: PathBuf,
    screenshot_path: PathBuf,
    osz_path: PathBuf,
    r2: Option<Arc<R2Manager>>,
}

impl Storage {
    pub async fn new(
        replay_path: PathBuf,
        screenshot_path: PathBuf,
        osz_path: PathBuf,
        bucket_name: &str,
        cloudflare_kv_uri: &str,
        cloudflare_kv_client_id: &str,
        cloudflare_kv_secret: &str,
    ) -> Self {
        let r2 = if bucket_name != "none" && !bucket_name.is_empty() {
            Some(Arc::new(
                R2Manager::new(
                    bucket_name,
                    cloudflare_kv_uri,
                    cloudflare_kv_client_id,
                    cloudflare_kv_secret,
                )
                .await,
            ))
        } else {
            None
        };

        Self {
            replay_path,
            screenshot_path,
            osz_path,
            r2,
        }
    }

    fn osz_file(&self, mapset_id: i32) -> PathBuf {
        self.osz_path.join(format!("{mapset_id}.osz"))
    }
    fn replay_file(&self, score_id: u64) -> PathBuf {
        self.replay_path.join(format!("{score_id}.osr"))
    }
    fn screenshot_file(&self, name_with_ext: &str) -> PathBuf {
        self.screenshot_path.join(name_with_ext)
    }

    fn osz_key(&self, mapset_id: i32) -> String {
        format!("osz/{mapset_id}.osz")
    }
    fn replay_key(&self, score_id: u64) -> String {
        format!("osr/{score_id}.osr")
    }
    fn screenshot_key(&self, name_with_ext: &str) -> String {
        format!("ss/{name_with_ext}")
    }

    pub async fn save_osz(&self, mapset_id: i32, data: &[u8]) -> Result<()> {
        if let Some(r2) = &self.r2 {
            r2.upload(
                &self.osz_key(mapset_id),
                data,
                None,
                Some("application/zip"),
            )
            .await;
        } else {
            fs::write(self.osz_file(mapset_id), data)?;
        }

        Ok(())
    }

    pub async fn load_osz(&self, mapset_id: i32) -> Result<Vec<u8>> {
        if let Some(r2) = &self.r2 {
            match r2.get(&self.osz_key(mapset_id)).await {
                Some(osz) => Ok(osz),
                None => Ok(fs::read(self.osz_file(mapset_id))?),
            }
        } else {
            Ok(fs::read(self.osz_file(mapset_id))?)
        }
    }

    pub async fn save_replay(&self, score_id: u64, frames: &[u8], lazer: Option<&[u8]>) -> Result<()> {
        let mut data = Vec::with_capacity(4 + frames.len() + lazer.map_or(0, |l| l.len()));
        // prefix: i32 frame length
        data.extend_from_slice(&(frames.len() as i32).to_le_bytes());
        data.extend_from_slice(frames);
        if let Some(l) = lazer {
            data.extend_from_slice(l);
        }
        if let Some(r2) = &self.r2 {
            r2.upload(
                &self.replay_key(score_id),
                &data,
                None,
                Some("application/octet-stream"),
            )
            .await;
        } else {
            fs::write(self.replay_file(score_id), &data)?;
        }
        Ok(())
    }

    pub async fn load_replay(&self, score_id: u64) -> Result<Vec<u8>> {
        let data = self.read_replay_raw(score_id).await?;
        // peel off 4-byte frame length prefix if present
        if data.len() >= 4 {
            let frame_len = i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
            if 4 + frame_len <= data.len() {
                return Ok(data[4..4 + frame_len].to_vec());
            }
        }
        // old format: whole file is frames
        Ok(data)
    }

    pub async fn load_replay_with_lazer(&self, score_id: u64) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
        let data = self.read_replay_raw(score_id).await?;
        if data.len() >= 4 {
            let frame_len = i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
            if 4 + frame_len <= data.len() {
                let frames = data[4..4 + frame_len].to_vec();
                let lazer = data[4 + frame_len..].to_vec();
                return Ok((frames, if lazer.is_empty() { None } else { Some(lazer) }));
            }
        }
        // old format
        Ok((data, None))
    }

    async fn read_replay_raw(&self, score_id: u64) -> Result<Vec<u8>> {
        if let Some(r2) = &self.r2 {
            match r2.get(&self.replay_key(score_id)).await {
                Some(replay) => Ok(replay),
                None => Ok(fs::read(self.replay_file(score_id))?),
            }
        } else {
            Ok(fs::read(self.replay_file(score_id))?)
        }
    }

    pub async fn save_screenshot(&self, name_with_ext: &str, data: &[u8]) -> Result<()> {
        if let Some(r2) = &self.r2 {
            let content_type = if name_with_ext.to_lowercase().ends_with(".jpeg") {
                "image/jpeg"
            } else {
                "image/png"
            };

            r2.upload(
                &self.screenshot_key(name_with_ext),
                data,
                None,
                Some(content_type),
            )
            .await;
        } else {
            fs::write(self.screenshot_file(name_with_ext), data)?;
        }

        Ok(())
    }

    pub async fn load_screenshot(&self, name_with_ext: &str) -> Result<Vec<u8>> {
        if let Some(r2) = &self.r2 {
            match r2.get(&self.screenshot_key(name_with_ext)).await {
                Some(ss) => Ok(ss),
                None => Ok(fs::read(self.screenshot_file(name_with_ext))?),
            }
        } else {
            Ok(fs::read(self.screenshot_file(name_with_ext))?)
        }
    }
}
