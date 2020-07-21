use crate::common::ast::*;

/// 式ノードの種類
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum ExpressionNodeKind {
    /// IF式ノード
    IF { cond_ex: ExNodeId, body: Vec<StNodeId>, alter: Option<Vec<StNodeId>> },

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
    MEMBER { id: ExNodeId, member: ExNodeId },

    /// 整数ノード
    INTEGER { value: i64 },
    /// 非符号付き整数ノード
    UINTEGER { value: u64 },
    /// 真偽値リテラル
    BOOLEAN { truth: bool },
    /// 文字列リテラル
    STRING { contents: String },
    /// 識別子ノード
    /// std::os::exit_with() みたいなのを["std", "os", "exit_with"] で保持
    IDENTIFIER { names: Vec<String> },
    /// 呼び出し式ノード
    CALL { names: Vec<String>, args: Vec<ExNodeId> },
}
