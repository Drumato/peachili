pub enum OperandSize {
    QWORD,
}

#[derive(Debug, Clone, Copy)]
pub struct Operand {
    kind: OperandKind,
}

impl Operand {
    pub fn to_atandt(&self) -> String {
        match &self.kind {
            OperandKind::REGISTER { reg } => reg.to_atandt(),
            OperandKind::IMMEDIATE { value } => format!("${}", value),
            OperandKind::MEMORY { base, offset } => {
                if *offset == 0 {
                    format!("({})", base.to_atandt())
                } else {
                    format!("-{}({})", offset, base.to_atandt())
                }
            }
        }
    }
    pub fn get_reg(&self) -> Register {
        match &self.kind {
            OperandKind::REGISTER { reg } => *reg,
            _ => unreachable!(),
        }
    }

    pub fn new(kind: OperandKind) -> Self {
        Self { kind }
    }

    pub fn get_kind(&self) -> &OperandKind {
        &self.kind
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OperandKind {
    IMMEDIATE { value: i64 },
    REGISTER { reg: Register },
    MEMORY { base: Register, offset: usize },
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Register {
    // 64bit general-purpose registers
    /// Accumulator Register
    RAX,

    /// (Stack) Base Pointer Register
    RBP,
    /// Stack Pointer Register
    RSP,
    /// Destination Index Register
    RDI,
    /// Source Index Register
    RSI,
    /// Data Register
    RDX,
    /// Counter Register
    RCX,
    /// Base Register
    RBX,

    // x64 appended registers
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Register {
    /// R10 ~ R15
    pub const AVAILABLES: usize = 6;

    pub fn to_str(&self) -> &'static str {
        match self {
            // 64bit general-purpose registers
            Register::RAX => "rax",
            Register::RCX => "rcx",
            Register::RDX => "rdx",
            Register::RBX => "rbx",
            Register::RSP => "rsp",
            Register::RBP => "rbp",
            Register::RSI => "rsi",
            Register::RDI => "rdi",
            Register::R8 => "r8",
            Register::R9 => "r9",
            Register::R10 => "r10",
            Register::R11 => "r11",
            Register::R12 => "r12",
            Register::R13 => "r13",
            Register::R14 => "r14",
            Register::R15 => "r15",
        }
    }

    pub fn to_atandt(&self) -> String {
        format!("%{}", self.to_str())
    }
}
