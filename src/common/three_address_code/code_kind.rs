use crate::common::three_address_code::ValueId;

/// Codeの種類
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum CodeKind {
    ADD {
        lop: ValueId,
        rop: ValueId,
        result: ValueId,
    },
    SUB {
        lop: ValueId,
        rop: ValueId,
        result: ValueId,
    },
    MUL {
        lop: ValueId,
        rop: ValueId,
        result: ValueId,
    },
    DIV {
        lop: ValueId,
        rop: ValueId,
        result: ValueId,
    },
    ASSIGN {
        value: ValueId,
        result: ValueId,
    },
    NEG {
        value: ValueId,
        result: ValueId,
    },
    ADDRESSOF {
        value: ValueId,
        result: ValueId,
    },
    DEREFERENCE {
        value: ValueId,
        result: ValueId,
    },
    MEMBER {
        id: ValueId,
        member: ValueId,
        result: ValueId,
    },
    RETURN {
        value: ValueId,
    },
}