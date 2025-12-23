use std::path::PathBuf;

#[derive(Clone)]
pub struct Storage {
    beatmap_path: PathBuf,
    replay_path: PathBuf,
    screenshot_path: PathBuf,
}

impl Storage {
    pub fn new(beatmap_path: PathBuf, replay_path: PathBuf, screenshot_path: PathBuf) -> Self {
        Self {
            beatmap_path,
            replay_path,
            screenshot_path,
        }
    }

    pub fn beatmap_file(&self, beatmap_id: i32) -> PathBuf {
        self.beatmap_path.join(format!("{beatmap_id}.osu"))
    }

    pub fn replay_file(&self, score_id: u64) -> PathBuf {
        self.replay_path.join(format!("{score_id}.osr"))
    }

    pub fn screenshot_file(&self, name_with_ext: &str) -> PathBuf {
        self.screenshot_path.join(name_with_ext)
    }
}
