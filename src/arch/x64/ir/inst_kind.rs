use crate::arch::x64::ir;

pub enum InstKind {
    // add[d/l/q] src, dst
    ADD {
        operand_size: ir::OperandSize,
        src: ir::Operand,
        dst: ir::Operand,
    },
    // sub[d/l/q] src, dst
    SUB {
        operand_size: ir::OperandSize,
        src: ir::Operand,
        dst: ir::Operand,
    },
    // mov[d/l/q] src, dst
    MOV {
        operand_size: ir::OperandSize,
        src: ir::Operand,
        dst: ir::Operand,
    },

    INLINEASM {
        contents: String,
    },
    CALL {
        name: String,
    },
    PUSH {
        operand_size: ir::OperandSize,
        value: ir::Operand,
    },
    POP {
        operand_size: ir::OperandSize,
        value: ir::Operand,
    },
    RET,
}

