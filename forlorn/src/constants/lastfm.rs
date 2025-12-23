use std::collections::HashMap;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct LastFmFlags: u32 {
        const CLEAN                        = 0;
        const SPEED_HACK_DETECTED           = 1 << 1;
        const INCORRECT_MOD_VALUE           = 1 << 2;
        const MULTIPLE_OSU_CLIENTS          = 1 << 3;
        const CHECKSUM_FAILURE              = 1 << 4;
        const FLASHLIGHT_CHECKSUM_INCORRECT = 1 << 5;
        const FLASHLIGHT_IMAGE_HACK         = 1 << 8;
        const PROCESS_INJECTED              = 1 << 9;
        const TRANSPARENT_WINDOW            = 1 << 10;
        const FAST_PRESS                    = 1 << 11;
        const RAW_MOUSE_DISCREPANCY          = 1 << 12;
        const RAW_KEYBOARD_DISCREPANCY       = 1 << 13;
        const RUN_WITH_LD_FLAG               = 1 << 14;
        const CONSOLE_OPEN                   = 1 << 15;
        const EXTRA_THREADS                  = 1 << 16;
        const HQ_ASSEMBLY                    = 1 << 17;
        const HQ_FILE                        = 1 << 18;
        const REGISTRY_EDITS                 = 1 << 19;
        const SDL2_LIBRARY                   = 1 << 20;
        const OPENSSL_LIBRARY                = 1 << 21;
        const AQN_MENU_SAMPLE                = 1 << 22;
        const INVALID_CHEAT_VALUES           = 1 << 23;
    }
}

impl LastFmFlags {
    pub fn explain(&self) -> Vec<String> {
        let map = Self::map();
        let raw = self.bits();

        let mut out = Vec::new();

        for i in 0..32 {
            let bit = 1u32 << i;
            if raw & bit == 0 {
                continue;
            }

            match map.get(&bit) {
                Some(msg) => out.push(msg.to_string()),
                None => out.push(format!("Undocumented Flag: 0x{:X}", bit)),
            }
        }

        out
    }

    fn map() -> HashMap<u32, &'static str> {
        HashMap::from([
            (
                LastFmFlags::SPEED_HACK_DETECTED.bits(),
                "[LIKELY] timewarp flag triggered (audio desync)",
            ),
            (
                LastFmFlags::INCORRECT_MOD_VALUE.bits(),
                "[MIXED] mod value mismatch (possible mod remover)",
            ),
            (
                LastFmFlags::MULTIPLE_OSU_CLIENTS.bits(),
                "[MIXED] multiple osu! clients open",
            ),
            (
                LastFmFlags::CHECKSUM_FAILURE.bits(),
                "[LIKELY] memory checksum mismatch",
            ),
            (
                LastFmFlags::FLASHLIGHT_CHECKSUM_INCORRECT.bits(),
                "[UNKNOWN] flashlight checksum failed",
            ),
            (
                LastFmFlags::FLASHLIGHT_IMAGE_HACK.bits(),
                "[CERTAIN] flashlight image manipulation detected",
            ),
            (
                LastFmFlags::PROCESS_INJECTED.bits(),
                "[MIXED] either our patcher's fault or osu! is injected", // most likely?
            ),
            (
                LastFmFlags::TRANSPARENT_WINDOW.bits(),
                "[LIKELY] transparent overlay window",
            ),
            (
                LastFmFlags::FAST_PRESS.bits(),
                "[LIKELY] abnormal fast key presses",
            ),
            (
                LastFmFlags::RAW_MOUSE_DISCREPANCY.bits(),
                "[LIKELY] raw mouse input mismatch",
            ),
            (
                LastFmFlags::RAW_KEYBOARD_DISCREPANCY.bits(),
                "[LIKELY] raw keyboard input mismatch",
            ),
            (
                LastFmFlags::RUN_WITH_LD_FLAG.bits(),
                "[MIXED] LD_PRELOAD / LD flags detected",
            ),
            (
                LastFmFlags::CONSOLE_OPEN.bits(),
                "[LIKELY] debugger / console attached",
            ),
            (
                LastFmFlags::EXTRA_THREADS.bits(),
                "[LIKELY] foreign threads detected",
            ),
            (
                LastFmFlags::HQ_ASSEMBLY.bits(),
                "[CERTAIN] HQOsu assembly detected",
            ),
            (LastFmFlags::HQ_FILE.bits(), "[MIXED] HQOsu files found"),
            (
                LastFmFlags::REGISTRY_EDITS.bits(),
                "[MIXED] registry edits detected",
            ),
            (
                LastFmFlags::SDL2_LIBRARY.bits(),
                "[CERTAIN] SDL2 cheat library detected",
            ),
            (
                LastFmFlags::OPENSSL_LIBRARY.bits(),
                "[CERTAIN] OpenSSL cheat library detected",
            ),
            (
                LastFmFlags::AQN_MENU_SAMPLE.bits(),
                "[CERTAIN] ancient AQN menu sound detected",
            ),
            (
                LastFmFlags::INVALID_CHEAT_VALUES.bits(),
                "[CERTAIN] invalid cheat values detected",
            ),
        ])
    }
}
