use super::expression;
/// 式ノード
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    // pub position: position::Position,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum StmtKind {
    Expr { expr: expression::Expr },
    Asm { insts: Vec<String> },
}
