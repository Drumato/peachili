use std::collections::HashMap;

use crate::compiler::common::frontend::{peachili_type, typed_ast};
pub struct Function {
    pub name: String,
    pub return_type: peachili_type::PeachiliType,
    pub params: HashMap<String, peachili_type::PeachiliType>,
    pub stack_size: usize,
    pub scope: typed_ast::Scope,
    pub stmts: Vec<typed_ast::Statement>,
}
