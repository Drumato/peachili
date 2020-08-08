use crate::arch::aarch64::ir;

pub struct Instruction {
    kind: ir::InstKind,
}

impl Instruction {
    pub fn to_assembly(&self) -> String {
        match &self.kind {
            ir::InstKind::ADD {
                operand_size,
                dst,
                lop,
                rop,
            } => match operand_size {
                ir::OperandSize::DWORD => format!(
                    "add {}, {}, {}",
                    dst.to_dword(),
                    lop.to_dword(),
                    rop.to_dword()
                ),
            },
            ir::InstKind::SUB {
                operand_size,
                dst,
                lop,
                rop,
            } => match operand_size {
                ir::OperandSize::DWORD => format!(
                    "sub {}, {}, {}",
                    dst.to_dword(),
                    lop.to_dword(),
                    rop.to_dword()
                ),
            },
            ir::InstKind::MUL {
                operand_size,
                dst,
                lop,
                rop,
            } => match operand_size {
                ir::OperandSize::DWORD => format!(
                    "mul {}, {}, {}",
                    dst.to_dword(),
                    lop.to_dword(),
                    rop.to_dword()
                ),
            },
            ir::InstKind::MOV {
                operand_size,
                dst,
                src,
            } => match operand_size {
                ir::OperandSize::DWORD => format!("mov {}, {}", dst.to_dword(), src.to_dword()),
            },
            ir::InstKind::STR {
                operand_size,
                dst,
                src,
            } => match operand_size {
                ir::OperandSize::DWORD => format!("str {}, {}", src.to_dword(), dst.to_dword()),
            },
            ir::InstKind::STP {
                operand_size,
                reg1,
                reg2,
                dst,
            } => match operand_size {
                ir::OperandSize::DWORD => format!(
                    "stp {}, {}, {}",
                    reg1.to_dword(),
                    reg2.to_dword(),
                    dst.to_dword()
                ),
            },
            ir::InstKind::LDP {
                operand_size,
                reg1,
                reg2,
                src,
            } => match operand_size {
                ir::OperandSize::DWORD => format!(
                    "ldp {}, {}, {}",
                    reg1.to_dword(),
                    reg2.to_dword(),
                    src.to_dword()
                ),
            },
            ir::InstKind::BL { name } => format!("bl \"{}\"", name),
            ir::InstKind::INLINEASM { contents } => contents.to_string(),
            ir::InstKind::RET => "ret".to_string(),
        }
    }

    pub fn new(k: ir::InstKind) -> Self {
        Self { kind: k }
    }
}
