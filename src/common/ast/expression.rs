use crate::common::ast::ExpressionNodeKind;
use crate::common::position;
use crate::common::token::TokenKind;

use id_arena::Id;

pub type ExNodeId = Id<ExpressionNode>;

/// 式ノード
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ExpressionNode {
    k: ExpressionNodeKind,
    p: position::Position,
}

#[allow(dead_code)]
impl ExpressionNode {
    fn new(k: ExpressionNodeKind, p: position::Position) -> Self {
        Self { k, p }
    }
    pub fn new_integer(int_value: i64, pos: position::Position) -> Self {
        Self::new(ExpressionNodeKind::INTEGER { value: int_value }, pos)
    }
    pub fn new_identifier(names: Vec<String>, pos: position::Position) -> Self {
        Self::new(ExpressionNodeKind::IDENTIFIER { names }, pos)
    }
    pub fn new_prefix_op(operator: &TokenKind, value: ExNodeId, pos: position::Position) -> Self {
        let nk = match operator {
            TokenKind::MINUS => ExpressionNodeKind::NEG { value },
            TokenKind::AMPERSAND => ExpressionNodeKind::ADDRESSOF { value },
            TokenKind::ASTERISK => ExpressionNodeKind::DEREFERENCE { value },
            _ => panic!("cannot create prefix-operation from {}", operator),
        };
        Self::new(nk, pos)
    }
    pub fn new_postfix_op(operator: &TokenKind, value: ExNodeId, pos: position::Position) -> Self {
        let nk = match operator {
            TokenKind::DOT => ExpressionNodeKind::MEMBER { value },
            _ => panic!("cannot create postfix-operation from {}", operator),
        };
        Self::new(nk, pos)
    }

    pub fn new_binop(
        tk: &TokenKind,
        lhs: ExNodeId,
        rhs: ExNodeId,
        pos: position::Position,
    ) -> Self {
        let nk = match tk {
            TokenKind::PLUS => ExpressionNodeKind::ADD { lhs, rhs },
            TokenKind::MINUS => ExpressionNodeKind::SUB { lhs, rhs },
            TokenKind::ASTERISK => ExpressionNodeKind::MUL { lhs, rhs },
            TokenKind::SLASH => ExpressionNodeKind::DIV { lhs, rhs },
            TokenKind::ASSIGN => ExpressionNodeKind::ASSIGN { lhs, rhs },
            _ => panic!("cannot create binary-operation from {}", tk),
        };

        Self::new(nk, pos)
    }

    pub fn get_kind(&self) -> &ExpressionNodeKind {
        &self.k
    }
}
