#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SubmissionStatus {
    Failed = 0,
    Best = 1,
    Submitted = 2,
}

impl SubmissionStatus {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum RankedStatus {
    Inactive = -3,
    NotSubmitted = -1,
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
