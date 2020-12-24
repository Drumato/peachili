use std::collections::HashMap;

use crate::compiler::arch::x64;
pub struct Function {
    pub func_name: String,
    pub return_type: x64::PeachiliType,
    pub params: HashMap<String, x64::PeachiliType>,
    pub stack_size: usize,
    pub local_variables: HashMap<String, FrameObject>,
    pub stmts: Vec<x64::Statement>,
}

pub struct FrameObject {
    pub stack_offset: usize,
    pub p_type: x64::PeachiliType,
}
