use crate::arch::aarch64::ir;

pub enum InstKind {
    /// Add
    ADD {
        operand_size: ir::OperandSize,
        dst: ir::Operand,
        lop: ir::Operand,
        rop: ir::Operand,
    },
    /// Sub
    SUB {
        operand_size: ir::OperandSize,
        dst: ir::Operand,
        lop: ir::Operand,
        rop: ir::Operand,
    },
    /// Mul
    MUL {
        operand_size: ir::OperandSize,
        dst: ir::Operand,
        lop: ir::Operand,
        rop: ir::Operand,
    },
    /// NEG
    NEG {
        operand_size: ir::OperandSize,
        dst: ir::Operand,
        value: ir::Operand,
    },
    /// Move
    MOV {
        operand_size: ir::OperandSize,
        dst: ir::Operand,
        src: ir::Operand,
    },
    /// Store
    STR {
        operand_size: ir::OperandSize,
        dst: ir::Operand,
        src: ir::Operand,
    },
    /// Store Register Pair
    STP {
        operand_size: ir::OperandSize,
        reg1: ir::Register,
        reg2: ir::Register,
        dst: ir::Operand,
    },
    /// Load To Register
    LDR {
        operand_size: ir::OperandSize,
        dst: ir::Operand,
        src: ir::Operand,
    },
    /// Load Register Pair
    LDP {
        operand_size: ir::OperandSize,
        reg1: ir::Register,
        reg2: ir::Register,
        src: ir::Operand,
    },

    /// Branch with Link.
    BL { name: String },

    /// Inline Assembly
    INLINEASM { contents: String },

    /// Return Value
    RET,
}
