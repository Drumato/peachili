use crate::common::peachili_type;
use crate::common::three_address_code::*;
use id_arena::{Arena, Id};
use std::sync::{Arc, Mutex};

pub type IRFunctionId = Id<IRFunction>;
pub type CodeArena = Arc<Mutex<Arena<code::Code>>>;
pub type ValueArena = Arc<Mutex<Arena<value::Value>>>;

/// 三番地コードにおける関数表現
#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub return_type: peachili_type::Type,
    pub codes: Vec<code::CodeId>,

    pub value_allocator: ValueArena,
    pub code_allocator: CodeArena,
    pub args: Vec<String>,
    // ベーシックブロック系の情報も持たせる
}

#[allow(dead_code)]
impl IRFunction {
    pub fn get_value(&self, v_id: ValueId) -> Value {
        self.value_allocator
            .lock()
            .unwrap()
            .get(v_id)
            .unwrap()
            .clone()
    }

    pub fn get_code(&self, c_id: CodeId) -> Code {
        self.code_allocator
            .lock()
            .unwrap()
            .get(c_id)
            .unwrap()
            .clone()
    }

    pub fn get_called_name(&self, v_id: ValueId) -> String {
        self.get_value(v_id).copy_contents()
    }
}
