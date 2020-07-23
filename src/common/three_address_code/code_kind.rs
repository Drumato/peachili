use crate::common::three_address_code::ValueId;
use crate::common::three_address_code::function::ValueArena;

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
}

impl CodeKind {
    fn unop(operator: &str, result: &ValueId, value: &ValueId, value_arena: ValueArena) -> String {
        let res = value_arena.lock().unwrap().get(*result).unwrap().clone().dump();
        let value = value_arena.lock().unwrap().get(*value).unwrap().clone().dump();

        let code_str = format!("\"{} <- {} {}\"", res, operator, value);
        let mut node_str = format!("{}[label = {}, shape=\"box\"]", res, code_str);

        if value.starts_with("temp") {
            node_str += &format!("\n    {} -> {}", value, res);
        }

        node_str
    }
    fn binop(operator: &str, result: &ValueId, lop: &ValueId, rop: &ValueId, value_arena: ValueArena) -> String {
        let res = value_arena.lock().unwrap().get(*result).unwrap().clone().dump();
        let lop = value_arena.lock().unwrap().get(*lop).unwrap().clone().dump();
        let rop = value_arena.lock().unwrap().get(*rop).unwrap().clone().dump();

        let code_str = format!("\"{} <- {} {} {}\"", res, lop, operator, rop);
        let mut node_str = format!("{}[label = {}, shape=\"box\"]", res, code_str);

        if lop.starts_with("temp") {
            node_str += &format!("\n    {} -> {}", lop, res);
        }
        if rop.starts_with("temp") {
            node_str += &format!("\n    {} -> {}", rop, res);
        }

        node_str
    }
    pub fn dump(&self, value_arena: ValueArena) -> String {
        match self {
            CodeKind::ADD { lop, rop, result } => {
                Self::binop("+", result, lop, rop, value_arena)
            }
            CodeKind::SUB { lop, rop, result } => {
                Self::binop("-", result, lop, rop, value_arena)
            }
            CodeKind::MUL { lop, rop, result } => {
                Self::binop("*", result, lop, rop, value_arena)
            }
            CodeKind::DIV { lop, rop, result } => {
                Self::binop("/", result, lop, rop, value_arena)
            }
            CodeKind::ASSIGN { value, result } => {
                Self::unop("", result, value, value_arena)
            }
            CodeKind::NEG { value, result } => {
                Self::unop("-", result, value, value_arena)
            }
            CodeKind::ADDRESSOF { value, result } => {
                Self::unop("&", result, value, value_arena)
            }
            CodeKind::DEREFERENCE { value, result } => {
                Self::unop("*", result, value, value_arena)
            }
            CodeKind::MEMBER { id, member, result } => {
                let res = value_arena.lock().unwrap().get(*result).unwrap().clone().dump();
                let id = value_arena.lock().unwrap().get(*id).unwrap().clone().dump();
                let member = value_arena.lock().unwrap().get(*member).unwrap().clone().dump();
                format!(
                    "\"{}\"[label = \"{} <- {}.{}\", shape=\"box\"]",
                    res,
                    res,
                    id,
                    member,
                )
            }
            CodeKind::RETURN { value } => {
                let ret_value = value_arena.lock().unwrap().get(*value).unwrap().clone().dump();
                format!(
                    "\"return {}\"[shape=\"box\"]\n    {} -> \"return {}\";",
                    ret_value,
                    ret_value,
                    ret_value
                )
            }
            CodeKind::PARAM { value } => {
                let arg_value = value_arena.lock().unwrap().get(*value).unwrap().clone().dump();
                format!(
                    "\"param {}\"[shape=\"box\"];",
                    arg_value,
                )
            }
            CodeKind::CALL { name, result } => {
                let result = value_arena.lock().unwrap().get(*result).unwrap().clone().dump();
                let name = value_arena.lock().unwrap().get(*name).unwrap().clone().dump();
                format!(
                    "\"{}\"[label = \"{} <- call {}\", shape=\"box\"]",
                    result,
                    result,
                    name
                )
            }
            CodeKind::ALLOC { temp } => {
                let allocated = value_arena.lock().unwrap().get(*temp).unwrap().clone().dump();
                format!(
                    "\"alloc {}\"[shape=\"box\"];",
                    allocated,
                )
            }
            CodeKind::LABEL { name } => {
                format!(
                    "\"label {}\"[shape=\"box\"];",
                    name,
                )
            }
            CodeKind::JUMPIFFALSE { label, cond_result } => {
                let cond = value_arena.lock().unwrap().get(*cond_result).unwrap().clone().dump();
                format!(
                    "\"jump {} if not {}\"[shape=\"box\"];",
                    label,
                    cond,
                )
            }
            CodeKind::JUMP { label } => {
                format!(
                    "\"jump {}\"[shape=\"box\"];",
                    label,
                )
            }
        }
    }
}