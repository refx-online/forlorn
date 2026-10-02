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

/// Per-mode rank status packed into one u64: 3 bits per mode id (0-15).
/// Statuses aren't contiguous (-2 is unused), so codes are mapped
/// explicitly: -3->0, -1->1, 0->2, 1->3, 2->4, 3->5, 4->6, 5->7.
pub const STATUS_BITS: u32 = 3;
const STATUS_CODE_MASK: u64 = 0b111;

/// All modes pending: every 3-bit group is 010.
pub const STATUS_MASK_ALL_PENDING: u64 = 0x492492492492;

fn status_code(status: i32) -> u64 {
    match status {
        -3 => 0,
        -1 => 1,
        0 => 2,
        1 => 3,
        2 => 4,
        3 => 5,
        4 => 6,
        5 => 7,
        _ => 2,
    }
}

fn code_status(code: u64) -> i32 {
    match code & STATUS_CODE_MASK {
        0 => -3,
        1 => -1,
        2 => 0,
        3 => 1,
        4 => 2,
        5 => 3,
        6 => 4,
        7 => 5,
        _ => 0,
    }
}

/// Read one mode's status out of a packed mask. Out-of-range modes read Pending.
pub fn status_at(mask: u64, mode: i32) -> i32 {
    if !(0..16).contains(&mode) {
        return RankedStatus::Pending.as_i32();
    }
    code_status(mask >> (mode as u32 * STATUS_BITS))
}

/// Write one mode's status into a packed mask.
pub fn with_status(mask: u64, mode: i32, status: i32) -> u64 {
    if !(0..16).contains(&mode) {
        return mask;
    }
    let code = status_code(status);
    let shift = mode as u32 * STATUS_BITS;
    (mask & !(STATUS_CODE_MASK << shift)) | (code << shift)
}

/// Same status for every mode (what the old global column meant).
pub fn all_modes_status(status: i32) -> u64 {
    let code = status_code(status);
    (0..16).fold(0u64, |mask, mode| mask | (code << (mode * STATUS_BITS)))
}
