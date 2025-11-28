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
    pub fn into_discord(&self) -> &'static str {
        match self {
            Grade::F => "<:grade_f:1251961173482405936>",
            Grade::D => "<:grade_d:1251961153874296832>",
            Grade::C => "<:grade_c:1251961155857940571>",
            Grade::B => "<:grade_b:1251961158483705936>",
            Grade::A => "<:grade_a:1251961160731721830>",
            Grade::S => "<:grade_s:1251961171335188551>",
            Grade::SH => "<:grade_sh:1251961168763945102>",
            Grade::X => "<:grade_ss:1251961166700216450>",
            Grade::XH => "<:grade_ssh:1251961164225581207>",
            _ => "",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "F" => Grade::F,
            "D" => Grade::D,
            "C" => Grade::C,
            "B" => Grade::B,
            "A" => Grade::A,
            "S" => Grade::S,
            "SH" => Grade::SH,
            "X" => Grade::X,
            "XH" => Grade::XH,
            _ => Grade::N,
        }
    }
}
