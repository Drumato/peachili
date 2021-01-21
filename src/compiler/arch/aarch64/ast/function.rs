use std::collections::HashMap;

use crate::compiler::arch::aarch64;
pub struct Function {
    pub name: String,
    pub return_type: aarch64::PeachiliType,
    pub params: HashMap<String, aarch64::PeachiliType>,
    pub stack_size: usize,
    pub local_variables: HashMap<String, FrameObject>,
    pub stmts: Vec<aarch64::Statement>,
}

pub struct FrameObject {
    pub stack_offset: usize,
    pub p_type: aarch64::PeachiliType,
}
