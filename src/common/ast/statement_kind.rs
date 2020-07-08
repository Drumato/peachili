use crate::common::ast::*;

/// 文ノードの種類
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum StatementNodeKind {
    /// return <expression>;
    RETURNSTMT { expr: ExNodeId },
    /// <expression>;
    EXPRSTMT { expr: ExNodeId },
    /// ifret <expression>;
    IFRETSTMT { expr: ExNodeId },
    /// declare <ident-name> <type-name>;
    DECLARESTMT {
        ident_name: String,
        type_name: String,
    },
}
