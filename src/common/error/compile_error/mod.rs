mod tokenize_error;

pub use tokenize_error::*;

use colored::*;

use std::fmt;

use crate::common::position::Position;

/// Compilerが発行するエラーを格納
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct CompileError<K: CompileErrorKind> {
    /// エラーの種類
    k: K,

    /// エラーの箇所
    p: Position,
}

impl<K: CompileErrorKind> CompileError<K> {
    pub fn new(kind: K, position: Position) -> Self {
        Self {
            k: kind,
            p: position,
        }
    }
    /// 標準エラー出力にエラーを出力する
    pub fn output(&self) {
        eprintln!("{}{} : {}", self.k.category().red().bold(), self.p, self.k);
    }
}

/// Compilerが発行するエラーの種類
pub trait CompileErrorKind: fmt::Display {
    /// 具体的なエラーの分類
    fn category(&self) -> &'static str;
}
