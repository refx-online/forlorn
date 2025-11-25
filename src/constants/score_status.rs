#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum SubmissionStatus {
    BEST = 1,
    SUBMITTED = 2,
}

impl SubmissionStatus {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(SubmissionStatus::BEST),
            2 => Some(SubmissionStatus::SUBMITTED),
            _ => None,
        }
    }
}
