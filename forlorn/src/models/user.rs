use serde::{Deserialize, Serialize};

use crate::constants::Privileges;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub safe_name: String,
    pub privilege: i32,
    pub pw_bcrypt: String,
    pub country: String,
    pub silence_end: i32,
    pub donor_end: i32,
    pub creation_time: i32,
    pub latest_activity: i32,
    pub clan_id: i32,
    pub clan_priv: i8,
    pub preferred_mode: i32,
    pub play_style: i32,
    pub custom_badge_name: Option<String>,
    pub custom_badge_icon: Option<String>,
    pub userpage_content: Option<String>,
    pub api_key: Option<String>,
    pub whitelist: i32,
    pub preferred_metric: String,
}

impl User {
    pub fn preferred_metric(&self) -> &str {
        &self.preferred_metric
    }

    pub fn restricted(&self) -> bool {
        !Privileges::from_bits_retain(self.privilege).contains(Privileges::UNRESTRICTED)
    }

    pub fn whitelist_stage(&self) -> usize {
        if self.privilege & Privileges::WHITELISTED.bits() != 0 {
            self.whitelist.clamp(1, 4) as usize
        } else {
            0
        }
    }
}
