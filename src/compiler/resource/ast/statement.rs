use crate::common::position as pos;
use crate::compiler::resource as res;

#[derive(Clone)]
pub struct StatementNode {
    pub kind: StatementNodeKind,
    position: pos::Position,
}

impl StatementNode {
    fn new(stmt_kind: StatementNodeKind, stmt_pos: pos::Position) -> Self {
        Self {
            kind: stmt_kind,
            position: stmt_pos,
        }
    }

    pub fn new_return(expr: res::ExpressionNode, cur_pos: pos::Position) -> Self {
        Self::new(StatementNodeKind::RETURN(Box::new(expr)), cur_pos)
    }

    pub fn new_vardecl(cur_pos: pos::Position) -> Self {
        Self::new(StatementNodeKind::VARDECL, cur_pos)
    }
}

impl std::fmt::Display for StatementNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.position, self.kind)
    }
}

#[derive(Clone)]
pub enum StatementNodeKind {
    RETURN(Box<res::ExpressionNode>),
    VARDECL,
}

impl std::fmt::Display for StatementNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::RETURN(inner) => write!(f, "return {}", inner),
            Self::VARDECL => write!(f, "(vardecl)"),
        }
    }
}
