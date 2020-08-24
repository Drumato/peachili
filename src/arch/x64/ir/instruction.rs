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
                dst,
            } => match operand_size {
                ir::OperandSize::QWORD => format!("addq {}, {}", src.to_atandt(), dst.to_atandt()),
            },
            ir::InstKind::SUB {
                operand_size,
                src,
                dst,
            } => match operand_size {
                ir::OperandSize::QWORD => format!("subq {}, {}", src.to_atandt(), dst.to_atandt()),
            },
            ir::InstKind::IMUL {
                operand_size,
                src,
                dst,
            } => match operand_size {
                ir::OperandSize::QWORD => format!("imulq {}, {}", src.to_atandt(), dst.to_atandt()),
            },
            ir::InstKind::IDIV {
                operand_size,
                value,
            } => match operand_size {
                ir::OperandSize::QWORD => format!("idivq {}", value.to_atandt()),
            },
            ir::InstKind::MOV {
                operand_size,
                src,
                dst,
            } => match operand_size {
                ir::OperandSize::QWORD => format!("movq {}, {}", src.to_atandt(), dst.to_atandt()),
            },
            ir::InstKind::CMP {
                operand_size,
                src,
                dst,
            } => match operand_size {
                ir::OperandSize::QWORD => format!("cmpq {}, {}", src.to_atandt(), dst.to_atandt()),
            },
            ir::InstKind::LEA {
                operand_size,
                src,
                dst,
            } => match operand_size {
                ir::OperandSize::QWORD => format!("leaq {}, {}", src.to_atandt(), dst.to_atandt()),
            },
            ir::InstKind::INLINEASM { contents } => contents.to_string(),
            ir::InstKind::CALL { name } => format!("call \"{}\"", name),
            ir::InstKind::PUSH {
                operand_size,
                value,
            } => match operand_size {
                ir::OperandSize::QWORD => format!("pushq {}", value.to_atandt()),
            },
            ir::InstKind::POP {
                operand_size,
                value,
            } => match operand_size {
                ir::OperandSize::QWORD => format!("popq {}", value.to_atandt()),
            },
            ir::InstKind::NEG {
                operand_size,
                value,
            } => match operand_size {
                ir::OperandSize::QWORD => format!("negq {}", value.to_atandt()),
            },
            ir::InstKind::CLTD => "cltd".to_string(),
            ir::InstKind::RET => "ret".to_string(),
            ir::InstKind::JMP { label } => format!("jmp .L{}", label),
            ir::InstKind::JE { label } => format!("je .L{}", label),
        }
    }

    pub fn new(k: ir::InstKind) -> Self {
        Self { kind: k }
    }
}
