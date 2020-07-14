use fmt::Formatter;
use std::fmt;
use crate::common::error::CompileErrorKind;

/// Analyzerが発行するエラーを格納
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TypeError {
    /// エラーの種類
    kind: TypeErrorKind,
}

/// Analyzerが発行するエラーの種類を列挙
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TypeErrorKind {
    /// 型を導出しきれなかった
    CANNOTRESOLVE { type_name: String },
}

impl CompileErrorKind for TypeErrorKind {
    fn category(&self) -> &'static str {
        "TypeError"
    }
}

impl fmt::Display for TypeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            TypeErrorKind::CANNOTRESOLVE { type_name } => {
                format!("cannot resolve a type -> `{}`", type_name)
            }
        };

        write!(f, "{}", s)
    }
}
