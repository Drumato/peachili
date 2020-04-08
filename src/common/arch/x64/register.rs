#[allow(dead_code)]
pub enum Reg64 {
    RAX,
    RBP,
    RSP,
}

impl Reg64 {
    fn to_str(&self) -> &'static str {
        match self {
            Self::RAX => "rax",
            Self::RBP => "rbp",
            Self::RSP => "rsp",
        }
    }
}

impl std::fmt::Display for Reg64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
