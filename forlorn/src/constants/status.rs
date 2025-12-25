#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SubmissionStatus {
    Quit = -1,
    Failed = 0,
    Submitted = 1,
    Best = 2,
}

impl SubmissionStatus {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    pub fn from_i32(status: i32) -> Self {
        match status {
            -1 => SubmissionStatus::Quit,
            0 => SubmissionStatus::Failed,
            1 => SubmissionStatus::Submitted,
            2 => SubmissionStatus::Best,
            _ => SubmissionStatus::Failed,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SubmissionStatus::Quit => "Quit",
            SubmissionStatus::Failed => "Failed",
            SubmissionStatus::Submitted => "Submitted",
            SubmissionStatus::Best => "Best",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum RankedStatus {
    Pending = 0,
    UpdateAvailable = 1,
    Ranked = 2,
    Approved = 3,
    Qualified = 4,
    Loved = 5,
}

impl RankedStatus {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    pub fn as_osu_api(&self) -> i32 {
        match self {
            RankedStatus::Pending => 0,
            RankedStatus::Ranked => 1,
            RankedStatus::Approved => 2,
            RankedStatus::Qualified => 3,
            RankedStatus::Loved => 4,
            RankedStatus::UpdateAvailable => -2,
        }
    }

    pub fn from_osudirect(osudirect_status: i32) -> Self {
        match osudirect_status {
            0 | 7 => RankedStatus::Ranked,
            2 | 5 => RankedStatus::Pending,
            3 => RankedStatus::Qualified,
            // 4: all ranked statuses
            8 => RankedStatus::Loved,
            _ => RankedStatus::UpdateAvailable,
        }
    }
}
