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
    /// 代入ノード
    ASSIGN { lhs: ExNodeId, rhs: ExNodeId },

    /// 符号反転
    NEG { value: ExNodeId },
    /// アドレッシング
    ADDRESSOF { value: ExNodeId },
    /// デリファレンス
    DEREFERENCE { value: ExNodeId },
    /// メンバアクセス
    MEMBER { value: ExNodeId },

    /// 整数ノード
    INTEGER { value: i64 },
    /// 識別子ノード
    /// std::os::exit_with() みたいなのを["std", "os", "exit_with"] で保持
    IDENTIFIER { names: Vec<String> },
}
