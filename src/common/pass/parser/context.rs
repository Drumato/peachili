use crate::common::analyze_resource::ast;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// パース処理に必要な情報を集約する構造体
pub struct Context {
    pub fn_arena: ast::FnArena,
    pub called_functions: HashSet<String>,
    pub module_name: String,
    pub stmt_arena: ast::StmtArena,
    pub expr_arena: ast::ExprArena,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            fn_arena: Arc::new(Mutex::new(Default::default())),
            called_functions: HashSet::new(),
            module_name: String::new(),
            stmt_arena: Arc::new(Mutex::new(Default::default())),
            expr_arena: Arc::new(Mutex::new(Default::default())),
        }
    }
}
