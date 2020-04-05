use crate::common::position as pos;

#[allow(dead_code)]
pub struct ExpressionNode {
    kind: ExpressionNodeKind,
    position: pos::Position,
}

impl ExpressionNode {
    fn new(expr_kind: ExpressionNodeKind, expr_pos: pos::Position) -> Self {
        Self {
            kind: expr_kind,
            position: expr_pos,
        }
    }

    pub fn new_intlit(int_value: i128, cur_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::INTEGER(int_value), cur_pos)
    }
}

impl std::fmt::Display for ExpressionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

pub enum ExpressionNodeKind {
    INTEGER(i128),
}

impl std::fmt::Display for ExpressionNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::INTEGER(v) => write!(f, "{}", v),
        }
    }
}
