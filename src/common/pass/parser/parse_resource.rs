use crate::common::ast;
use std::sync::{Arc, Mutex};
use id_arena::Arena;

/// パーサの必要な資源の集約
/// トークン列を持たせると読みづらくなるので，持たせない．
pub struct ParseResource {
    pub stmt_arena: ast::StmtArena,
    pub expr_arena: ast::ExprArena,
    pub module_name: String,
}

impl ParseResource {
    pub fn new(module_name: String) -> Self {
        Self {
            stmt_arena: Arc::new(Mutex::new(Arena::new())),
            expr_arena: Arc::new(Mutex::new(Arena::new())),
            module_name,
        }
    }
}