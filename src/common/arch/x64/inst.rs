use crate::common::arch::x64::*;

#[allow(dead_code)]
pub struct Instruction {
    kind: InstKind,
}

impl Instruction {
    fn new(ik: InstKind) -> Self {
        Self { kind: ik }
    }

    pub fn to_intelcode(&self) -> String {
        match &self.kind {
            // immediate
            InstKind::PUSHINT64(v) => format!("push {}", v),

            // register
            InstKind::MOVREGTOREG64(dst, src) => format!("mov {}, {}", dst, src),
            InstKind::PUSHREG64(value) => format!("push {}", value),
            InstKind::POPREG64(value) => format!("pop {}", value),
            InstKind::SUBREGBYUSIZE(dst, value) => format!("sub {}, {}", dst, value),

            // etc
            InstKind::RET => "ret".to_string(),
            InstKind::COMMENT(contents) => format!("# {}", contents),
        }
    }

    // immediate
    pub fn pushint64(int_value: i128) -> Self {
        Self::new(InstKind::PUSHINT64(int_value))
    }
    // register
    pub fn movreg_toreg64(dst: Reg64, src: Reg64) -> Self {
        Self::new(InstKind::MOVREGTOREG64(dst, src))
    }
    pub fn pushreg64(reg: Reg64) -> Self {
        Self::new(InstKind::PUSHREG64(reg))
    }
    pub fn popreg64(reg: Reg64) -> Self {
        Self::new(InstKind::POPREG64(reg))
    }
    pub fn subreg_byusize(reg: Reg64, value: usize) -> Self {
        Self::new(InstKind::SUBREGBYUSIZE(reg, value))
    }

    // etc
    pub fn comment(contents: String) -> Self {
        Self::new(InstKind::COMMENT(contents))
    }
    pub fn ret() -> Self {
        Self::new(InstKind::RET)
    }
}

// TODO: i128 -> i64
#[allow(dead_code)]
pub enum InstKind {
    // immediate
    PUSHINT64(i128),

    // register
    MOVREGTOREG64(Reg64, Reg64),
    PUSHREG64(Reg64),
    POPREG64(Reg64),
    SUBREGBYUSIZE(Reg64, usize),

    // etc
    RET,
    COMMENT(String),
}
