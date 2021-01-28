use std::{cell::RefCell, rc::Rc};

use crate::compiler::common::frontend::ast;
/// 式ノード
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    // pub position: position::Position,
}

impl Stmt {
    pub fn new_edge(s: Self) -> Edge {
        Rc::new(RefCell::new(s))
    }
}
type Edge = Rc<RefCell<Stmt>>;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum StmtKind {
    HalfOpenCountup {
        block: Vec<Stmt>,
        id: String,
        from: ast::Expr,
        lessthan: ast::Expr,
    },
    ClosedCountup {
        block: Vec<Stmt>,
        id: String,
        from: ast::Expr,
        to: ast::Expr,
    },
    Block {
        stmts: Vec<Stmt>,
    },
    Declare {
        var_name: String,
        type_name: Vec<String>,
    },
    Expr {
        expr: ast::Expr,
    },
    Asm {
        insts: Vec<String>,
    },
}
