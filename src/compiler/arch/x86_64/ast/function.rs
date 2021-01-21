use std::collections::HashMap;

use crate::compiler::arch::x86_64;
pub struct Function {
    pub name: String,
    pub return_type: x86_64::PeachiliType,
    pub params: HashMap<String, x86_64::PeachiliType>,
    pub stack_size: usize,
    pub local_variables: HashMap<String, FrameObject>,
    pub stmts: Vec<x86_64::Statement>,
}

pub struct FrameObject {
    pub stack_offset: usize,
    pub p_type: x86_64::PeachiliType,
}
