use crate::common::arch::x64::*;

#[allow(dead_code)]
pub struct Instruction {
    kind: InstKind,
}

impl Instruction {
    fn new(ik: InstKind) -> Self {
        Self { kind: ik }
    }

    pub fn get_kind(&self) -> &InstKind {
        &self.kind
    }

    pub fn to_at_code(&self) -> String {
        match &self.kind {
            // immediate
            InstKind::PUSHINT64(v) => format!("pushq {}", v.to_at()),
            InstKind::PUSHUINT64(v) => format!("pushq {}", v.to_at()),
            InstKind::MOVIMMTOREG64(value, dst) => {
                format!("movq {}, {}", value.to_at(), dst.to_at())
            }
            InstKind::SUBREGBYUINT64(value, dst) => {
                format!("subq {}, {}", value.to_at(), dst.to_at())
            }

            // register
            InstKind::ADDREGTOREG64(src, dst) => format!("addq {}, {}", src.to_at(), dst.to_at()),
            InstKind::SUBREGTOREG64(src, dst) => format!("subq {}, {}", src.to_at(), dst.to_at()),
            InstKind::CMPREGANDINT64(v, reg) => format!("cmpq {}, {}", v.to_at(), reg.to_at()),
            InstKind::CMPREGANDREG64(r1, r2) => format!("cmpq {}, {}", r1.to_at(), r2.to_at()),
            InstKind::IMULREGTOREG64(src, dst) => format!("imulq {}, {}", src.to_at(), dst.to_at()),
            InstKind::IDIVREG64(value) => format!("idivq {}", value.to_at()),

            InstKind::MOVREGTOREG64(src, dst) => format!("movq {}, {}", src.to_at(), dst.to_at()),
            InstKind::MOVREGTOMEM64(src, base_reg, offset) => {
                format!("movq {}, -{}({})", src.to_at(), offset, base_reg.to_at())
            }
            InstKind::MOVMEMTOREG64(base_reg, offset, src) => {
                format!("movq -{}({}), {}", offset, base_reg.to_at(), src.to_at())
            }
            InstKind::INCREG64(value) => format!("incq {}", value.to_at()),
            InstKind::PUSHREG64(value) => format!("pushq {}", value.to_at()),
            InstKind::POPREG64(value) => format!("popq {}", value.to_at()),
            InstKind::NEGREG64(value) => format!("negq {}", value.to_at()),

            // etc
            InstKind::LEASTRINGADDRESSTOTREG64(label, reg, _addend) => {
                format!("leaq {}, {}", label, reg.to_at())
            }
            InstKind::LABEL(label) => format!("{}:", label),
            InstKind::JUMP(label) => format!("jmp {}", label),
            InstKind::JUMPEQUAL(label) => format!("je {}", label),
            InstKind::CALL(name) => format!("call {}", name),
            InstKind::CLTD => "cltd".to_string(),
            InstKind::RET => "ret".to_string(),
            InstKind::COMMENT(contents) => format!("# {}", contents),
            InstKind::SYSCALL => "syscall".to_string(),
        }
    }

    // immediate
    pub fn pushint64(int_value: i64) -> Self {
        Self::new(InstKind::PUSHINT64(Immediate::new_int64(int_value)))
    }
    pub fn pushuint64(int_value: u64) -> Self {
        Self::new(InstKind::PUSHUINT64(Immediate::new_uint64(int_value)))
    }

    // register
    pub fn addreg_toreg64(src: Reg64, dst: Reg64) -> Self {
        Self::new(InstKind::ADDREGTOREG64(src, dst))
    }
    pub fn cmpreg_andint64(imm: i64, reg: Reg64) -> Self {
        Self::new(InstKind::CMPREGANDINT64(Immediate::new_int64(imm), reg))
    }
    pub fn cmpreg_andreg64(r1: Reg64, r2: Reg64) -> Self {
        Self::new(InstKind::CMPREGANDREG64(r1, r2))
    }
    pub fn subreg_toreg64(src: Reg64, dst: Reg64) -> Self {
        Self::new(InstKind::SUBREGTOREG64(src, dst))
    }
    pub fn imulreg_toreg64(src: Reg64, dst: Reg64) -> Self {
        Self::new(InstKind::IMULREGTOREG64(src, dst))
    }
    pub fn idivreg64(value: Reg64) -> Self {
        Self::new(InstKind::IDIVREG64(value))
    }
    pub fn movreg_toreg64(src: Reg64, dst: Reg64) -> Self {
        Self::new(InstKind::MOVREGTOREG64(src, dst))
    }
    pub fn movreg_tomem64(src: Reg64, base_reg: Reg64, offset: usize) -> Self {
        Self::new(InstKind::MOVREGTOMEM64(src, base_reg, offset))
    }
    pub fn movmem_toreg64(base_reg: Reg64, offset: usize, src: Reg64) -> Self {
        Self::new(InstKind::MOVMEMTOREG64(base_reg, offset, src))
    }
    pub fn movimm_toreg64(value: i64, dst: Reg64) -> Self {
        Self::new(InstKind::MOVIMMTOREG64(Immediate::new_int64(value), dst))
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
    pub fn negreg64(value: Reg64) -> Self {
        Self::new(InstKind::NEGREG64(value))
    }
    pub fn increg64(value: Reg64) -> Self {
        Self::new(InstKind::INCREG64(value))
    }

    // etc
    pub fn lea_string_addr_to_reg(label: String, reg: Reg64, addend: u64) -> Self {
        Self::new(InstKind::LEASTRINGADDRESSTOTREG64(label, reg, addend))
    }
    pub fn jump_label(label: String) -> Self {
        Self::new(InstKind::JUMP(label))
    }
    pub fn jump_equal_label(label: String) -> Self {
        Self::new(InstKind::JUMPEQUAL(label))
    }
    pub fn label(label: String) -> Self {
        Self::new(InstKind::LABEL(label))
    }
    pub fn call(sym: String) -> Self {
        Self::new(InstKind::CALL(sym))
    }
    pub fn cltd() -> Self {
        Self::new(InstKind::CLTD)
    }
    pub fn comment(contents: String) -> Self {
        Self::new(InstKind::COMMENT(contents))
    }
    pub fn ret() -> Self {
        Self::new(InstKind::RET)
    }
    pub fn syscall() -> Self {
        Self::new(InstKind::SYSCALL)
    }
}

#[allow(dead_code)]
pub enum InstKind {
    // ***************
    // *  Immediate  *
    // ***************

    // push
    PUSHINT64(Immediate),
    PUSHUINT64(Immediate),

    // ****************
    // *   Register   *
    // ****************

    // add
    ADDREGTOREG64(Reg64, Reg64),
    // cmp
    CMPREGANDINT64(Immediate, Reg64),
    CMPREGANDREG64(Reg64, Reg64),
    // sub
    SUBREGBYUINT64(Immediate, Reg64),
    SUBREGTOREG64(Reg64, Reg64),
    // imul
    IMULREGTOREG64(Reg64, Reg64),
    // idiv
    IDIVREG64(Reg64),
    // mov
    MOVREGTOREG64(Reg64, Reg64),
    MOVREGTOMEM64(Reg64, Reg64, usize),
    MOVMEMTOREG64(Reg64, usize, Reg64),
    MOVIMMTOREG64(Immediate, Reg64),
    // push
    PUSHREG64(Reg64),
    // pop
    POPREG64(Reg64),
    // neg
    NEGREG64(Reg64),
    // inc
    INCREG64(Reg64),

    // etc
    LEASTRINGADDRESSTOTREG64(String, Reg64, u64),
    LABEL(String),
    JUMP(String),
    JUMPEQUAL(String),
    CALL(String),
    CLTD,
    RET,
    SYSCALL,
    COMMENT(String),
}
