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
    pub fn from_params(mode: i32, mods: Mods) -> GameMode {
        // i dont even know
        if mode >= 4 {
            return match mode {
                4 => GameMode::RX_OSU,
                5 => GameMode::RX_TAIKO,
                6 => GameMode::RX_CATCH,
                8 => GameMode::AP_OSU,
                12 => GameMode::CHEAT_OSU,
                16 => GameMode::CHEAT_CHEAT_OSU,
                20 => GameMode::TOUCH_DEVICE_OSU,
                _ => GameMode::VN_OSU,
            };
        }

        if mods.contains(Mods::TOUCHSCREEN) && mode == 0 {
            return GameMode::TOUCH_DEVICE_OSU;
        } else if mods.contains(Mods::AUTOPILOT) && mode == 0 {
            return GameMode::AP_OSU;
        } else if mods.contains(Mods::RELAX) && mode != 3 {
            return match mode {
                0 => GameMode::RX_OSU,
                1 => GameMode::RX_TAIKO,
                2 => GameMode::RX_CATCH,
                _ => GameMode::VN_OSU,
            };
        }

        match mode {
            0 => GameMode::VN_OSU,
            1 => GameMode::VN_TAIKO,
            2 => GameMode::VN_CATCH,
            3 => GameMode::VN_MANIA,
            _ => GameMode::VN_OSU,
        }
    }

    pub fn cheat(self) -> bool {
        matches!(self, GameMode::CHEAT_OSU | GameMode::CHEAT_CHEAT_OSU)
    }

    pub fn ngeki_nkatu(self) -> bool {
        matches!(
            self,
            GameMode::VN_TAIKO | GameMode::RX_TAIKO | GameMode::VN_MANIA
        )
    }

    pub fn as_vanilla(self) -> i32 {
        self as i32 % 4
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }

    pub fn as_str(self) -> &'static str {
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

    /// The PP cap thresholds for each game mode,
    /// used for whitelist stage 0–5.
    ///
    /// This logic is a portion of the python codebase.
    ///
    /// TODO: move this somewhere else
    ///
    /// NOTE: * = estimation
    ///
    ///                0     1     2     3     4*    5*
    /// - vn!std:    900 , 1000, 1300, 1500, 1700, 2300
    /// - vn!taiko:  1000, 1800, 2000, 2400, 2800, 3100
    /// - vn!catch:  1100, 1800, 2000, 2400, 2800, 3100
    /// - vn!mania:  1600, 1800, 2000, 2400, 2800, 3100
    /// - rx!std:    1600, 1800, 2000, 2400, 2800, 4700
    /// - rx!taiko:  1000, 1800, 2000, 2400, 2800, 4200
    /// - rx!catch:  1000, 1800, 2000, 2400, 2800, 4200
    /// - ap!std:    1200, 1800, 2000, 2400, ∞
    pub fn pp_cap(&self) -> &'static [i32] {
        match self {
            GameMode::VN_OSU => &[900, 1000, 1300, 1500, 1700, 2300],
            GameMode::VN_TAIKO => &[1000, 1800, 2000, 2400, 2800, 3100],
            GameMode::VN_CATCH => &[1100, 1800, 2000, 2400, 2800, 3100],
            GameMode::VN_MANIA => &[1600, 1800, 2000, 2400, 2800, 3100],

            GameMode::RX_OSU => &[1600, 1800, 2000, 2400, 2800, 4700],
            GameMode::RX_TAIKO => &[1000, 1800, 2000, 2400, 2800, 4200],
            GameMode::RX_CATCH => &[1000, 1800, 2000, 2400, 2800, 4200],

            GameMode::AP_OSU => &[1200, 1800, 2000, 2400, i32::MAX], // ∞

            // well, we doesn't have to check for cheats and touch device
            _ => &[],
        }
    }
}
