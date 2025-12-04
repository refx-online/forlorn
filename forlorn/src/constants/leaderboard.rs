#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderboardType {
    Local = 0,
    Top = 1,
    Mods = 2,
    Friends = 3,
    Country = 4,
}

impl LeaderboardType {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => LeaderboardType::Local,
            1 => LeaderboardType::Top,
            2 => LeaderboardType::Mods,
            3 => LeaderboardType::Friends,
            4 => LeaderboardType::Country,
            _ => LeaderboardType::Local,
        }
    }
}
