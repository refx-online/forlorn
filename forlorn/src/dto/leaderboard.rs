use serde::Deserialize;

use crate::constants::{GameMode, Mods};

#[derive(Debug, Deserialize)]
pub struct GetScores {
    #[serde(rename = "us")]
    pub username: String,

    #[serde(rename = "ha")]
    pub password_md5: String,

    #[serde(rename = "s")]
    pub requesting_from_editor: i32,

    #[allow(unused)]
    #[serde(rename = "vv")]
    pub leaderboard_version: i32,

    #[serde(rename = "v")]
    pub leaderboard_type: i32,

    #[serde(rename = "c")]
    pub map_md5: String,

    #[serde(rename = "f")]
    pub map_filename: String,

    #[serde(rename = "m")]
    pub mode: i32,

    #[serde(rename = "i")]
    pub map_set_id: i32,

    #[serde(rename = "mods")]
    pub mods: i32,

    #[allow(unused)]
    #[serde(rename = "h")]
    pub map_package_hash: Option<String>, // todo?

    #[serde(rename = "a")]
    pub aqn_files_found: i32,

    #[serde(rename = "fx", default)]
    pub is_refx: i32,
}

impl GetScores {
    pub fn requesting_from_editor(&self) -> bool {
        self.requesting_from_editor != 0
    }

    pub fn aqn_files_found(&self) -> bool {
        self.aqn_files_found != 0
    }

    pub fn is_refx(&self) -> bool {
        self.is_refx != 0
    }

    pub fn mode(&self) -> GameMode {
        GameMode::from_params(self.mode, self.mods())
    }

    pub fn mods(&self) -> Mods {
        Mods::from_bits_truncate(self.mods)
    }
}
