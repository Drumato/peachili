use crate::common::arch::x64::*;

#[allow(dead_code)]
pub struct Instruction {
    kind: InstKind,
}

impl Instruction {
    fn new(ik: InstKind) -> Self {
        Self { kind: ik }
    }

    pub fn to_at_code(&self) -> String {
        match &self.kind {
            // immediate
            InstKind::PUSHINT64(v) => format!("pushq {}", v.to_at()),

            // register
            InstKind::MOVREGTOREG64(src, dst) => format!("movq {}, {}", src.to_at(), dst.to_at()),
            InstKind::PUSHREG64(value) => format!("pushq {}", value.to_at()),
            InstKind::POPREG64(value) => format!("popq {}", value.to_at()),
            InstKind::SUBREGBYUINT64(value, dst) => {
                format!("subq {}, {}", value.to_at(), dst.to_at())
            }

            // etc
            InstKind::RET => "ret".to_string(),
            InstKind::COMMENT(contents) => format!("# {}", contents),
        }
    }

    // immediate
    pub fn pushint64(int_value: i64) -> Self {
        Self::new(InstKind::PUSHINT64(Immediate::new_int64(int_value)))
    }
    // register
    pub fn movreg_toreg64(src: Reg64, dst: Reg64) -> Self {
        Self::new(InstKind::MOVREGTOREG64(src, dst))
    }
    pub fn pushreg64(reg: Reg64) -> Self {
        Self::new(InstKind::PUSHREG64(reg))
    }
    pub fn popreg64(reg: Reg64) -> Self {
        Self::new(InstKind::POPREG64(reg))
    }
    pub fn subreg_byuint64(value: u64, reg: Reg64) -> Self {
        Self::new(InstKind::SUBREGBYUINT64(Immediate::new_uint64(value), reg))
    }

    // etc
    pub fn comment(contents: String) -> Self {
        Self::new(InstKind::COMMENT(contents))
    }
    pub fn ret() -> Self {
        Self::new(InstKind::RET)
    }
}

#[allow(dead_code)]
pub enum InstKind {
    // immediate
    PUSHINT64(Immediate),

    // register
    MOVREGTOREG64(Reg64, Reg64),
    PUSHREG64(Reg64),
    POPREG64(Reg64),
    SUBREGBYUINT64(Immediate, Reg64),

    // etc
    RET,
    COMMENT(String),
}
