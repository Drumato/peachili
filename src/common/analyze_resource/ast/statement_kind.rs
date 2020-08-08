use crate::common::ast::{ExNodeId, StNodeId};

/// 文ノードの種類
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum StatementNodeKind {
    /// "return" expression `;`
    RETURN { expr: ExNodeId },
    /// expression `;`
    EXPR { expr: ExNodeId },
    /// "ifret" expression `;`
    IFRET { expr: ExNodeId },
    /// "declare" identifier type `;`
    DECLARE {
        ident_name: String,
        type_name: String,
    },
    /// "countup" identifier "begin" expression "exclude" expression block `;`
    COUNTUP {
        ident_name: String,
        begin_ex: ExNodeId,
        endpoint_ex: ExNodeId,
        body: Vec<StNodeId>,
    },
    /// "asm" block `;
    ASM { stmts: Vec<StNodeId> },
    /// "varinit" identifier type `=` expression `;`
    VARINIT {
        ident_name: String,
        type_name: String,
        expr: ExNodeId,
    },
    /// "const" identifier type `=` expression `;`
    CONST {
        ident_name: String,
        type_name: String,
        expr: ExNodeId,
    },
}
