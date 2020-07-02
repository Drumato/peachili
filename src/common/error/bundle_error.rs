use colored::*;

use std::fmt;
use fmt::Formatter;

/// Bundlerが発行するエラーを格納
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct BundleError {
    /// エラーの種類
    kind: BundleErrorKind
}

impl BundleError {
    pub fn new(kind: BundleErrorKind) -> Self {
        Self {
            kind
        }
    }
    /// 標準エラー出力にエラーを出力する
    pub fn output(&self) {
        eprintln!("{} : {}", "BundleError".red().bold(), self.kind);
    }
}

/// Bundlerが発行するエラーの種類を列挙
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BundleErrorKind {
    /// import しているファイルが存在しない
    NOTFOUNDSUCHAFILE {
        file_name: String,
    }
}

impl fmt::Display for BundleErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            BundleErrorKind::NOTFOUNDSUCHAFILE { file_name } => format!("not found such a file -> `{}`", file_name),
        };

        write!(f, "{}", s)
    }
}