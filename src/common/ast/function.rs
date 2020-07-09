use crate::common::{
    ast::*,
    position,
};
use id_arena::Id;
use indexmap::IndexMap;

pub type FnId = Id<Function>;

/// 関数
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub stmts: Vec<StNodeId>,
    pub return_type: String,

    /// arg_name -> arg_type
    pub args: IndexMap<String, String>,
    pub pos: position::Position,
}