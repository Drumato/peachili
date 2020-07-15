use crate::common::{
    ast::*,
    position,
};
use id_arena::{Id, Arena};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

pub type FnId = Id<Function>;
pub type StmtArena = Arc<Mutex<Arena<StatementNode>>>;
pub type ExprArena = Arc<Mutex<Arena<ExpressionNode>>>;

/// 関数
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub stmts: Vec<StNodeId>,
    pub return_type: String,

    /// arg_name -> arg_type
    pub args: BTreeMap<String, String>,
    pub pos: position::Position,

    pub module_name: String,

    // アロケータ
    pub stmt_arena: StmtArena,
    pub expr_arena: ExprArena,
}