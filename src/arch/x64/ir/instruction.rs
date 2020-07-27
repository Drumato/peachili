use crate::arch::x64::ir;

pub struct Instruction {
    kind: ir::InstKind,
}

impl Instruction {
    pub fn to_atandt(&self) -> String {
        match &self.kind {
            ir::InstKind::ADD {
                operand_size,
                src,
                dst
            } => match operand_size {
                ir::OperandSize::QWORD => format!("addq {}, {}", src.to_atandt(), dst.to_atandt()),
            },
            ir::InstKind::SUB {
                operand_size,
                src,
                dst
            } => match operand_size {
                ir::OperandSize::QWORD => format!("subq {}, {}", src.to_atandt(), dst.to_atandt()),
            },
            ir::InstKind::MOV {
                operand_size,
                src,
                dst
            } => match operand_size {
                ir::OperandSize::QWORD => format!("movq {}, {}", src.to_atandt(), dst.to_atandt()),
            },
            ir::InstKind::INLINEASM { contents } => contents.
                to_string(),
            ir::InstKind::CALL { name } => format!("call \"{}\"", name),
            ir::InstKind::PUSH { operand_size, value } => match operand_size {
                ir::OperandSize::QWORD => format!("pushq {}", value.to_atandt()),
            },
            ir::InstKind::POP { operand_size, value } => match operand_size {
                ir::OperandSize::QWORD => format!("popq {}", value.to_atandt()),
            },
            ir::InstKind::RET => "ret".to_string(),
        }
    }

    pub fn new(k: ir::InstKind) -> Self {
        Self {
            kind: k,
        }
    }
}