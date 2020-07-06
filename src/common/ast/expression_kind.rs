use crate::common::ast::*;

/// 式ノードの種類
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum ExpressionNodeKind {
    /// 加算ノード
    ADD { lhs: ExNodeId, rhs: ExNodeId },
    /// 減算ノード
    SUB { lhs: ExNodeId, rhs: ExNodeId },
    /// 乗算ノード
    MUL { lhs: ExNodeId, rhs: ExNodeId },
    /// 除算ノード
    DIV { lhs: ExNodeId, rhs: ExNodeId },

    /// 整数ノード
    INTEGER { value: i64 },
}
