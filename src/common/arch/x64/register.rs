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
    pub fn to_at(&self) -> String {
        format!("%{}", self.to_str())
    }
}
