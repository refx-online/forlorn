use crate::constants::mods::Mods;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum GameMode {
    VN_OSU = 0,
    VN_TAIKO = 1,
    VN_CATCH = 2,
    VN_MANIA = 3,

    RX_OSU = 4,
    RX_TAIKO = 5,
    RX_CATCH = 6,

    AP_OSU = 8,

    CHEAT_OSU = 12,
    CHEAT_CHEAT_OSU = 16,
    TOUCH_DEVICE_OSU = 20,
}

impl GameMode {
    pub fn from_params(mode: u8, mods: Mods) -> GameMode {
        if mods.contains(Mods::TOUCHSCREEN) && mode == GameMode::VN_OSU as u8 {
            return GameMode::TOUCH_DEVICE_OSU;
        } else if mods.contains(Mods::AUTOPILOT) && mode == GameMode::VN_OSU as u8 {
            return GameMode::AP_OSU;
        } else if mods.contains(Mods::RELAX) && mode != GameMode::VN_MANIA as u8 {
            return match mode {
                0 => GameMode::RX_OSU,
                1 => GameMode::RX_TAIKO,
                2 => GameMode::RX_CATCH,
                12 => GameMode::CHEAT_OSU,
                16 => GameMode::CHEAT_CHEAT_OSU,
                _ => GameMode::VN_OSU,
            };
        }

        match mode {
            0 => GameMode::VN_OSU,
            1 => GameMode::VN_TAIKO,
            2 => GameMode::VN_CATCH,
            3 => GameMode::VN_MANIA,
            12 => GameMode::CHEAT_OSU,
            16 => GameMode::CHEAT_CHEAT_OSU,
            _ => GameMode::VN_OSU,
        }
    }

    pub fn as_vanilla(self) -> u8 {
        self as u8 % 4
    }

    pub fn ngeki_nkatu(self) -> bool {
        matches!(
            self,
            GameMode::VN_TAIKO | GameMode::VN_MANIA | GameMode::RX_TAIKO
        )
    }

    /// TODO: create a macro to handle cases like this
    pub fn repr(self) -> &'static str {
        match self {
            GameMode::VN_OSU => "vn!std",
            GameMode::VN_TAIKO => "vn!taiko",
            GameMode::VN_CATCH => "vn!catch",
            GameMode::VN_MANIA => "vn!mania",

            GameMode::RX_OSU => "rx!std",
            GameMode::RX_TAIKO => "rx!taiko",
            GameMode::RX_CATCH => "rx!catch",

            GameMode::AP_OSU => "ap!std",

            GameMode::CHEAT_OSU => "cheat!std",
            GameMode::CHEAT_CHEAT_OSU => "cheatcheat!std",

            GameMode::TOUCH_DEVICE_OSU => "td!std",
        }
    }
}
