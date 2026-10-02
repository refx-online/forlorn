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

    AP_OSU = 7,

    CHEAT_OSU = 8,
    CHEAT_TAIKO = 9,
    CHEAT_CATCH = 10,
    CHEAT_MANIA = 11,

    CHEAT_RX_OSU = 12,
    CHEAT_RX_TAIKO = 13,
    CHEAT_RX_CATCH = 14,

    CHEAT_AP_OSU = 15,
}

impl GameMode {
    pub fn from_params(mode: i32, mods: Mods) -> GameMode {
        // explicit cheat-rx/ap bytes win outright
        match mode {
            12 => return GameMode::CHEAT_RX_OSU,
            13 => return GameMode::CHEAT_RX_TAIKO,
            14 => return GameMode::CHEAT_RX_CATCH,
            15 => return GameMode::CHEAT_AP_OSU,
            _ => {}
        }

        // legacy cheatcheat byte folds into the cheat group
        let mode = if mode == 16 { 8 } else { mode };

        // touch folds into vanilla std (the TD mod bit stays on the score).
        // i dont even know
        if mode == 20 {
            return GameMode::VN_OSU;
        }

        if mode >= 4 {
            return match mode {
                4 => GameMode::RX_OSU,
                5 => GameMode::RX_TAIKO,
                6 => GameMode::RX_CATCH,
                7 => GameMode::AP_OSU,
                8 => GameMode::CHEAT_OSU,
                9 => GameMode::CHEAT_TAIKO,
                10 => GameMode::CHEAT_CATCH,
                11 => GameMode::CHEAT_MANIA,
                _ => GameMode::VN_OSU,
            };
        }

        if mods.contains(Mods::AUTOPILOT) && mode == 0 {
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

    /// Cheat classification for score submission, where the raw 12/16 byte
    /// carries no game mode — take it from the played map instead.
    /// Explicit bytes (8-15) already carry it and pass through.
    pub fn from_cheat_submission(raw: i32, beatmap_mode: i32, mods: Mods) -> GameMode {
        if (8..=15).contains(&raw) {
            return GameMode::from_params(raw, mods);
        }

        let base = beatmap_mode.clamp(0, 3);
        if mods.contains(Mods::AUTOPILOT) && base == 0 {
            return GameMode::CHEAT_AP_OSU;
        }
        if mods.contains(Mods::RELAX) && base != 3 {
            return match base {
                0 => GameMode::CHEAT_RX_OSU,
                1 => GameMode::CHEAT_RX_TAIKO,
                2 => GameMode::CHEAT_RX_CATCH,
                _ => GameMode::CHEAT_OSU,
            };
        }
        match base {
            1 => GameMode::CHEAT_TAIKO,
            2 => GameMode::CHEAT_CATCH,
            3 => GameMode::CHEAT_MANIA,
            _ => GameMode::CHEAT_OSU,
        }
    }

    pub fn cheat(self) -> bool {
        matches!(
            self,
            GameMode::CHEAT_OSU
                | GameMode::CHEAT_TAIKO
                | GameMode::CHEAT_CATCH
                | GameMode::CHEAT_MANIA
                | GameMode::CHEAT_RX_OSU
                | GameMode::CHEAT_RX_TAIKO
                | GameMode::CHEAT_RX_CATCH
                | GameMode::CHEAT_AP_OSU
        )
    }

    /// Lenient validation tier (old cheatcheat rules). Strict tier is the
    /// plain cheat non-rx group.
    pub fn cheat_lenient(self) -> bool {
        matches!(
            self,
            GameMode::CHEAT_RX_OSU
                | GameMode::CHEAT_RX_TAIKO
                | GameMode::CHEAT_RX_CATCH
                | GameMode::CHEAT_AP_OSU
        )
    }

    pub fn ngeki_nkatu(self) -> bool {
        matches!(
            self,
            GameMode::VN_TAIKO
                | GameMode::RX_TAIKO
                | GameMode::VN_MANIA
                | GameMode::CHEAT_TAIKO
                | GameMode::CHEAT_RX_TAIKO
                | GameMode::CHEAT_MANIA
        )
    }

    /// NOTE: explicit match, never derive it — ap ids would land on
    /// the wrong game (7 % 4 == 3 == mania, but ap is std-only).
    pub fn as_vanilla(self) -> i32 {
        match self {
            GameMode::VN_OSU | GameMode::RX_OSU | GameMode::AP_OSU | GameMode::CHEAT_OSU => 0,
            GameMode::VN_TAIKO | GameMode::RX_TAIKO | GameMode::CHEAT_TAIKO => 1,
            GameMode::VN_CATCH | GameMode::RX_CATCH | GameMode::CHEAT_CATCH => 2,
            GameMode::VN_MANIA | GameMode::CHEAT_MANIA => 3,
            GameMode::CHEAT_RX_OSU | GameMode::CHEAT_AP_OSU => 0,
            GameMode::CHEAT_RX_TAIKO => 1,
            GameMode::CHEAT_RX_CATCH => 2,
        }
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
            GameMode::CHEAT_TAIKO => "cheat!taiko",
            GameMode::CHEAT_CATCH => "cheat!catch",
            GameMode::CHEAT_MANIA => "cheat!mania",

            GameMode::CHEAT_RX_OSU => "cheat-rx!std",
            GameMode::CHEAT_RX_TAIKO => "cheat-rx!taiko",
            GameMode::CHEAT_RX_CATCH => "cheat-rx!catch",

            GameMode::CHEAT_AP_OSU => "cheat-ap!std",
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
