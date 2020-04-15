#[allow(dead_code)]
#[derive(Clone)]
pub enum Reg64 {
    RAX,
    RBP,
    RSP,
    RDI,
    RSI,
    RDX,
    RCX,
    R8,
    R9,
}

impl Reg64 {
    fn to_str(&self) -> &'static str {
        match self {
            Self::RAX => "rax",
            Self::RBP => "rbp",
            Self::RSP => "rsp",
            Self::RDI => "rdi",
            Self::RSI => "rsi",
            Self::RDX => "rdx",
            Self::RCX => "rcx",
            Self::R8 => "r8",
            Self::R9 => "r9",
        }
    }
    pub fn to_at(&self) -> String {
        format!("%{}", self.to_str())
    }
}
