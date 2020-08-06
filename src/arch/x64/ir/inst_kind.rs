use crate::arch::x64::ir;

pub enum InstKind {
    /// add[d/l/q] src, dst
    ADD {
        operand_size: ir::OperandSize,
        src: ir::Operand,
        dst: ir::Operand,
    },
    /// sub[d/l/q] src, dst
    SUB {
        operand_size: ir::OperandSize,
        src: ir::Operand,
        dst: ir::Operand,
    },
    /// imul[d/l/q] src, dst
    IMUL {
        operand_size: ir::OperandSize,
        src: ir::Operand,
        dst: ir::Operand,
    },
    /// idiv[d/l/q] src, dst
    IDIV {
        operand_size: ir::OperandSize,
        value: ir::Operand,
    },
    /// mov[d/l/q] src, dst
    MOV {
        operand_size: ir::OperandSize,
        src: ir::Operand,
        dst: ir::Operand,
    },
    /// cmp[d/l/q] src, dst
    CMP {
        operand_size: ir::OperandSize,
        src: ir::Operand,
        dst: ir::Operand,
    },
    /// lea[d/l/q] src, dst
    LEA {
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
    NEG {
        operand_size: ir::OperandSize,
        value: ir::Operand,
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
    CLTD,
    JMP {
        label: String,
    },
    JE {
        label: String,
    },
}

