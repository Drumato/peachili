use crate::common::{ast::StatementNodeKind, position};

use id_arena::Id;

pub type StNodeId = Id<StatementNode>;

/// 文ノード
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct StatementNode {
    k: StatementNodeKind,
    p: position::Position,
}

#[allow(dead_code)]
impl StatementNode {
    pub fn new(k: StatementNodeKind, p: position::Position) -> Self {
        Self { k, p }
    }
    pub fn get_kind(&self) -> &StatementNodeKind {
        &self.k
    }
    pub fn is_ifret(&self) -> bool {
        match self.k {
            StatementNodeKind::IFRET { expr: _ } => true,
            _ => false,
        }
    }
}
