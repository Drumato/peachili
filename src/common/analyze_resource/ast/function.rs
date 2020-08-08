use crate::common::{
    ast::{ExpressionNode, StNodeId, StatementNode},
    position,
};
use id_arena::{Arena, Id};
use std::sync::{Arc, Mutex};

pub type FnId = Id<Function>;
pub type StmtArena = Arc<Mutex<Arena<StatementNode>>>;
pub type ExprArena = Arc<Mutex<Arena<ExpressionNode>>>;
pub type FnArena = Arc<Mutex<Arena<Function>>>;

/// 関数
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub stmts: Vec<StNodeId>,

    pub pos: position::Position,

    pub module_name: String,

    pub fn_type: FunctionTypeDef,

    // アロケータ
    pub stmt_arena: StmtArena,
    pub expr_arena: ExprArena,
}

impl Function {
    pub fn full_path(&self) -> String {
        if self.module_name.is_empty() {
            return self.name.to_string();
        }

        format!("{}::{}", self.module_name, self.name)
    }

    pub fn copy_return_type(&self) -> String {
        self.fn_type.return_type.clone()
    }

    pub fn get_parameters(&self) -> &Vec<(String, String)> {
        &self.fn_type.args
    }
}

#[derive(Debug, Clone)]
pub struct FunctionTypeDef {
    /// return type of the function
    pub return_type: String,

    /// arg_name -> arg_type
    pub args: Vec<(String, String)>,
}

impl FunctionTypeDef {
    pub fn new(return_type: String, args: Vec<(String, String)>) -> Self {
        Self { return_type, args }
    }
}
