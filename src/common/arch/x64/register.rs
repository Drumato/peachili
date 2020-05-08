pub const REX_PREFIX_BASE: u8 = 0x40;
pub const REX_PREFIX_WBIT: u8 = 0x08;
pub const REX_PREFIX_RBIT: u8 = 0x04;
// pub const REX_PREFIX_XBIT: u8 = 0x02;
pub const REX_PREFIX_BBIT: u8 = 0x01;

pub const MODRM_REGISTER_DISPLACEMENT8: u8 = 0x40;
pub const _MODRM_REGISTER_DISPLACEMENT32: u8 = 0x80;
pub const MODRM_REGISTER_REGISTER: u8 = 0xc0;

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
    RIP,
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
            Self::RIP => "rip",
        }
    }
    pub fn to_at(&self) -> String {
        format!("%{}", self.to_str())
    }
    pub fn from_at_str(reg_str: &str) -> Self {
        match reg_str {
            "%rax" => Self::RAX,
            "%rbp" => Self::RBP,
            "%rsp" => Self::RSP,
            "%rdi" => Self::RDI,
            "%rsi" => Self::RSI,
            "%rdx" => Self::RDX,
            "%rcx" => Self::RCX,
            "%r8" => Self::R8,
            "%r9" => Self::R9,
            "%rip" => Self::RIP,
            _ => panic!("undefined such a register -> {}", reg_str),
        }
    }

    pub fn machine_number(&self) -> u8 {
        match self {
            Self::RAX | Self::R8 => 0,
            Self::RCX | Self::R9 => 1,
            Self::RDX => 2,
            Self::RSP => 4,
            Self::RBP => 5,
            Self::RSI => 6,
            Self::RDI => 7,
            Self::RIP => 0b101,
        }
    }

    pub fn is_expanded(&self) -> bool {
        match self {
            Self::R8 | Self::R9 => true,
            Self::RAX
            | Self::RBP
            | Self::RSP
            | Self::RDI
            | Self::RSI
            | Self::RDX
            | Self::RCX
            | Self::RIP => false,
        }
    }
}
