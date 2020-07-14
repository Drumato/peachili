use fmt::Formatter;
use std::fmt;
use crate::common::error::CompileErrorKind;

/// Bundlerが発行するエラーを格納
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct BundleError {
    /// エラーの種類
    kind: BundleErrorKind,
}


/// Bundlerが発行するエラーの種類を列挙
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BundleErrorKind {
    /// import しているファイルが存在しない
    NOTFOUNDSUCHAFILE { file_name: String },
}


impl CompileErrorKind for BundleErrorKind {
    fn category(&self) -> &'static str {
        "BundleError"
    }
}

impl fmt::Display for BundleErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            BundleErrorKind::NOTFOUNDSUCHAFILE { file_name } => {
                format!("not found such a file -> `{}`", file_name)
            }
        };

        write!(f, "{}", s)
    }
}
