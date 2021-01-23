use std::collections::HashMap;

use crate::compiler::common::frontend::{peachili_type, typed_ast};
pub struct Function {
    pub name: String,
    pub return_type: peachili_type::PeachiliType,
    pub params: HashMap<String, peachili_type::PeachiliType>,
    pub stack_size: usize,
    pub local_variables: HashMap<String, FrameObject>,
    pub stmts: Vec<typed_ast::Statement>,
}

pub struct FrameObject {
    pub stack_offset: usize,
    pub p_type: peachili_type::PeachiliType,
}
