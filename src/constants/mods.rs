use bitflags::bitflags;
use std::collections::HashMap;

bitflags! {
    #[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct Mods: u32 {
        const NOMOD        = 0;
        const NOFAIL       = 1 << 0;
        const EASY         = 1 << 1;
        const TOUCHSCREEN  = 1 << 2;
        const HIDDEN       = 1 << 3;
        const HARDROCK     = 1 << 4;
        const SUDDENDEATH  = 1 << 5;
        const DOUBLETIME   = 1 << 6;
        const RELAX        = 1 << 7;
        const HALFTIME     = 1 << 8;
        const NIGHTCORE    = 1 << 9;
        const FLASHLIGHT   = 1 << 10;
        const AUTOPLAY     = 1 << 11;
        const SPUNOUT      = 1 << 12;
        const AUTOPILOT    = 1 << 13;
        const PERFECT      = 1 << 14;
        const KEY4         = 1 << 15;
        const KEY5         = 1 << 16;
        const KEY6         = 1 << 17;
        const KEY7         = 1 << 18;
        const KEY8         = 1 << 19;
        const FADEIN       = 1 << 20;
        const RANDOM       = 1 << 21;
        const CINEMA       = 1 << 22;
        const TARGET       = 1 << 23;
        const KEY9         = 1 << 24;
        const KEYCOOP      = 1 << 25;
        const KEY1         = 1 << 26;
        const KEY3         = 1 << 27;
        const KEY2         = 1 << 28;
        const SCOREV2      = 1 << 29;
        const MIRROR       = 1 << 30;
    }
}

pub fn repr(mods: Mods) -> String {
    if mods.is_empty() {
        return "NM".into();
    }

    let mut out = String::new();
    let dict = mod2modstr_dict();

    for (flag, s) in dict.iter() {
        if mods.contains(*flag) {
            out.push_str(s);
        }
    }

    out
}

pub fn mod2modstr_dict() -> HashMap<Mods, &'static str> {
    [
        (Mods::NOFAIL, "NF"),
        (Mods::EASY, "EZ"),
        (Mods::TOUCHSCREEN, "TD"),
        (Mods::HIDDEN, "HD"),
        (Mods::HARDROCK, "HR"),
        (Mods::SUDDENDEATH, "SD"),
        (Mods::DOUBLETIME, "DT"),
        (Mods::RELAX, "RX"),
        (Mods::HALFTIME, "HT"),
        (Mods::NIGHTCORE, "NC"),
        (Mods::FLASHLIGHT, "FL"),
        (Mods::AUTOPLAY, "AT"),
        (Mods::SPUNOUT, "SO"),
        (Mods::AUTOPILOT, "AP"),
        (Mods::PERFECT, "PF"),
        (Mods::KEY4, "K4"),
        (Mods::KEY5, "K5"),
        (Mods::KEY6, "K6"),
        (Mods::KEY7, "K7"),
        (Mods::KEY8, "K8"),
        (Mods::FADEIN, "FI"),
        (Mods::RANDOM, "RD"),
        (Mods::CINEMA, "CN"),
        (Mods::TARGET, "TP"),
        (Mods::KEY9, "K9"),
        (Mods::KEYCOOP, "KC"),
        (Mods::KEY1, "K1"),
        (Mods::KEY3, "K3"),
        (Mods::KEY2, "K2"),
        (Mods::SCOREV2, "V2"),
        (Mods::MIRROR, "MR"),
    ]
    .iter()
    .cloned()
    .collect()
}
