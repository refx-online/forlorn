#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum Grade {
    N = 0,
    F = 1,
    D = 2,
    C = 3,
    B = 4,
    A = 5,
    S = 6,
    SH = 7,
    X = 8,
    XH = 9,
}

impl Grade {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}
