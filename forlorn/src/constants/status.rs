#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SubmissionStatus {
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
            0 => SubmissionStatus::Failed,
            1 => SubmissionStatus::Submitted,
            2 => SubmissionStatus::Best,
            _ => SubmissionStatus::Failed,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
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
}
