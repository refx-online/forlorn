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
    RX_MANIA = 7, // unused

    AP_OSU = 8,
    AP_TAIKO = 9,  // unused
    AP_CATCH = 10, // unused
    AP_MANIA = 11, // unused

    CHEAT_OSU = 12,
    CHEAT_CHEAT_OSU = 16,
    TOUCH_DEVICE_OSU = 20,
}

impl GameMode {
    pub fn from_params(mode_vn: u8, mods: Mods) -> GameMode {
        if mods.contains(Mods::TOUCHSCREEN) && mode_vn == GameMode::VN_OSU as u8 {
            return GameMode::TOUCH_DEVICE_OSU;
        } else if mods.contains(Mods::AUTOPILOT) && mode_vn == GameMode::VN_OSU as u8 {
            return GameMode::AP_OSU;
        } else if mods.contains(Mods::RELAX) && mode_vn != GameMode::VN_MANIA as u8 {
            return match mode_vn {
                0 => GameMode::RX_OSU,
                1 => GameMode::RX_TAIKO,
                2 => GameMode::RX_CATCH,
                _ => GameMode::VN_OSU,
            };
        }

        match mode_vn {
            0 => GameMode::VN_OSU,
            1 => GameMode::VN_TAIKO,
            2 => GameMode::VN_CATCH,
            3 => GameMode::VN_MANIA,
            _ => GameMode::VN_OSU,
        }
    }

    pub fn as_vanilla(self) -> u8 {
        self as u8 % 4
    }
}

pub const GAMEMODE_REPR: [&str; 21] = [
    "vn!std",
    "vn!taiko",
    "vn!catch",
    "vn!mania",
    "rx!std",
    "rx!taiko",
    "rx!catch",
    "rx!mania",
    "ap!std",
    "ap!taiko",
    "ap!catch",
    "ap!mania",
    "cheat!std",
    "cheat!taiko",
    "cheat!catch",
    "cheat!mania",
    "cheatcheat!std",
    "cheatcheat!taiko",
    "cheatcheat!catch",
    "cheatcheat!mania",
    "td!std",
];
