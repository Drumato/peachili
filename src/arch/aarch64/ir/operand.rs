pub enum OperandSize {
    /// 64bit size
    DWORD,
}

#[derive(Debug, Clone, Copy)]
pub struct Operand {
    kind: OperandKind,
}

#[allow(dead_code)]
impl Operand {
    pub fn to_dword(&self) -> String {
        match &self.kind {
            OperandKind::REGISTER { reg } => reg.to_dword(),
            OperandKind::IMMEDIATE { value } => format!("#{}", value),
            OperandKind::MEMORY { base, offset } => {
                if *offset == 0 {
                    format!("[{}]", base.to_dword())
                } else {
                    format!("[{}, #{}]", base.to_dword(), *offset)
                }
            }
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
    MEMORY { base: Register, offset: isize },
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Register {
    GPR { number: usize },

    // Stack Pointer
    SP,
    // Frame Pointer
    FP,
    // Dword Link Register
    LINK,
}

impl Register {
    /// X8 ~ X18
    pub const AVAILABLES: usize = 10;

    pub fn to_dword(&self) -> String {
        match self {
            Register::SP => "sp".to_string(),
            Register::FP => "x29".to_string(),
            Register::LINK => "x30".to_string(),
            Register::GPR { number } => format!("x{}", number),
        }
    }
}