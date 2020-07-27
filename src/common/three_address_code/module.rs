use crate::common::three_address_code::*;
use id_arena::Arena;

#[derive(Debug, Clone)]
pub struct IRModule {
    pub funcs: Vec<function::IRFunctionId>,
    pub fn_allocator: Arena<function::IRFunction>,
}

impl Default for IRModule {
    fn default() -> Self {
        Self {
            funcs: Vec::new(),
            fn_allocator: Arena::new(),
        }
    }
}

impl IRModule {
    pub fn get_fn(&self, fn_id: &function::IRFunctionId) -> &function::IRFunction {
        self.fn_allocator.get(*fn_id).unwrap()
    }
}