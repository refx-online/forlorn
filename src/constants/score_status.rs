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

    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(SubmissionStatus::Best),
            2 => Some(SubmissionStatus::Submitted),
            _ => None,
        }
    }
}
