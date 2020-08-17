use crate::common::three_address_code::function::ValueArena;
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
    STORE {
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
        member: String,
        result: ValueId,
    },
    RETURN {
        value: ValueId,
    },
    PARAM {
        value: ValueId,
    },
    CALL {
        name: ValueId,
        result: ValueId,
    },
    ALLOC {
        temp: ValueId,
    },
    LABEL {
        name: String,
    },
    JUMPIFFALSE {
        label: String,
        cond_result: ValueId,
    },
    JUMP {
        label: String,
    },
    ASM {
        value: ValueId,
    },
}

impl CodeKind {
    fn unop(operator: &str, result: &ValueId, value: &ValueId, value_arena: ValueArena) -> String {
        let res = value_arena
            .lock()
            .unwrap()
            .get(*result)
            .unwrap()
            .clone()
            .dump();
        let value = value_arena
            .lock()
            .unwrap()
            .get(*value)
            .unwrap()
            .clone()
            .dump();

        format!("{} <- {} {}", res, operator, value)
    }
    fn binop(
        operator: &str,
        result: &ValueId,
        lop: &ValueId,
        rop: &ValueId,
        value_arena: ValueArena,
    ) -> String {
        let res = value_arena
            .lock()
            .unwrap()
            .get(*result)
            .unwrap()
            .clone()
            .dump();
        let lop = value_arena
            .lock()
            .unwrap()
            .get(*lop)
            .unwrap()
            .clone()
            .dump();
        let rop = value_arena
            .lock()
            .unwrap()
            .get(*rop)
            .unwrap()
            .clone()
            .dump();

        format!("{} <- {} {} {}", res, lop, operator, rop)
    }
    pub fn dump(&self, value_arena: ValueArena) -> String {
        match self {
            CodeKind::ADD { lop, rop, result } => Self::binop("+", result, lop, rop, value_arena),
            CodeKind::SUB { lop, rop, result } => Self::binop("-", result, lop, rop, value_arena),
            CodeKind::MUL { lop, rop, result } => Self::binop("*", result, lop, rop, value_arena),
            CodeKind::DIV { lop, rop, result } => Self::binop("/", result, lop, rop, value_arena),
            CodeKind::ASSIGN { value, result } => Self::unop("", result, value, value_arena),
            CodeKind::STORE { value, result } => {
                let result = value_arena
                    .lock()
                    .unwrap()
                    .get(*result)
                    .unwrap()
                    .clone()
                    .dump();
                let value = value_arena
                    .lock()
                    .unwrap()
                    .get(*value)
                    .unwrap()
                    .clone()
                    .dump();
                format!("store {} into {}", value, result)
            }
            CodeKind::NEG { value, result } => Self::unop("-", result, value, value_arena),
            CodeKind::ADDRESSOF { value, result } => Self::unop("&", result, value, value_arena),
            CodeKind::DEREFERENCE { value, result } => Self::unop("*", result, value, value_arena),
            CodeKind::MEMBER { id, member, result } => {
                let res = value_arena
                    .lock()
                    .unwrap()
                    .get(*result)
                    .unwrap()
                    .clone()
                    .dump();
                let id = value_arena.lock().unwrap().get(*id).unwrap().clone().dump();

                format!("{} <- {}.{}", res, id, member)
            }
            CodeKind::RETURN { value } => {
                let ret_value = value_arena
                    .lock()
                    .unwrap()
                    .get(*value)
                    .unwrap()
                    .clone()
                    .dump();
                format!("return {}", ret_value)
            }
            CodeKind::PARAM { value } => {
                let arg_value = value_arena
                    .lock()
                    .unwrap()
                    .get(*value)
                    .unwrap()
                    .clone()
                    .dump();
                format!("param {}", arg_value)
            }
            CodeKind::CALL { name, result } => {
                let result = value_arena
                    .lock()
                    .unwrap()
                    .get(*result)
                    .unwrap()
                    .clone()
                    .dump();
                let name = value_arena
                    .lock()
                    .unwrap()
                    .get(*name)
                    .unwrap()
                    .clone()
                    .dump();
                format!("{} <- call {}", result, name)
            }
            CodeKind::ALLOC { temp } => {
                let allocated = value_arena
                    .lock()
                    .unwrap()
                    .get(*temp)
                    .unwrap()
                    .clone()
                    .dump();
                format!("alloc {}", allocated,)
            }
            CodeKind::LABEL { name } => format!("label {}", name,),
            CodeKind::JUMPIFFALSE { label, cond_result } => {
                let cond = value_arena
                    .lock()
                    .unwrap()
                    .get(*cond_result)
                    .unwrap()
                    .clone()
                    .dump();
                format!("jump {} if not {}", label, cond,)
            }
            CodeKind::JUMP { label } => format!("jump {}", label,),
            CodeKind::ASM { value } => {
                let v = value_arena
                    .lock()
                    .unwrap()
                    .get(*value)
                    .unwrap()
                    .clone()
                    .dump();
                format!("asm {}", v)
            }
        }
    }
}
