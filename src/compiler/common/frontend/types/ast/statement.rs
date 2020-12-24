use super::expression;
/// 式ノード
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct StmtInfo<'a> {
    pub kind: StmtKind<'a>,
    // pub position: position::Position,
}

pub type Stmt<'a> = &'a StmtInfo<'a>;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum StmtKind<'a> {
    Expr { expr: expression::ExprInfo<'a> },
    Asm { insts: Vec<String> },
}
