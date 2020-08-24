use crate::common::ast::{ExpressionNodeKind, StNodeId};
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
    pub fn get_pos(&self) -> position::Position {
        self.p
    }
    pub fn copy_names(&self) -> Vec<String> {
        match self.get_kind() {
            ExpressionNodeKind::IDENTIFIER { names } => names.clone(),
            _ => panic!("cannot copy_names with not identifier"),
        }
    }
    pub fn new_integer(int_value: i64, pos: position::Position) -> Self {
        Self::new(ExpressionNodeKind::INTEGER { value: int_value }, pos)
    }
    pub fn new_identifier(names: Vec<String>, pos: position::Position) -> Self {
        Self::new(ExpressionNodeKind::IDENTIFIER { names }, pos)
    }
    pub fn new_uinteger(uint_value: u64, pos: position::Position) -> Self {
        Self::new(ExpressionNodeKind::UINTEGER { value: uint_value }, pos)
    }
    pub fn new_string_literal(contents: String, pos: position::Position) -> Self {
        Self::new(ExpressionNodeKind::STRING { contents }, pos)
    }
    pub fn new_boolean(truth: bool, pos: position::Position) -> Self {
        Self::new(ExpressionNodeKind::BOOLEAN { truth }, pos)
    }
    pub fn new_call(names: Vec<String>, args: Vec<ExNodeId>, pos: position::Position) -> Self {
        Self::new(ExpressionNodeKind::CALL { names, args }, pos)
    }
    pub fn new_if(
        cond_id: ExNodeId,
        body: Vec<StNodeId>,
        alter: Option<Vec<StNodeId>>,
        pos: position::Position,
    ) -> Self {
        Self::new(
            ExpressionNodeKind::IF {
                cond_ex: cond_id,
                body,
                alter,
            },
            pos,
        )
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
    pub fn new_postfix_op(
        operator: &TokenKind,
        id: ExNodeId,
        member: String,
        pos: position::Position,
    ) -> Self {
        let nk = match operator {
            TokenKind::DOT => ExpressionNodeKind::MEMBER { id, member },
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

    pub fn is_identifier(&self) -> bool {
        match self.get_kind() {
            ExpressionNodeKind::IDENTIFIER { names: _ } => true,
            _ => false,
        }
    }
    pub fn is_integer_literal(&self) -> bool {
        match self.get_kind() {
            ExpressionNodeKind::INTEGER { value: _ } => true,
            _ => false,
        }
    }
    pub fn get_integer_value(&self) -> i64 {
        match self.get_kind() {
            ExpressionNodeKind::INTEGER { value } => *value,
            _ => panic!("cannot get value from {:?}", self),
        }
    }
}
