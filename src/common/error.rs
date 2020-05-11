extern crate colored;

use colored::*;

use crate::common::{option as opt, position as pos};

#[derive(Clone)]
pub struct CompileError {
    kind: CmpErrorKind,
    position: pos::Position,
}

impl CompileError {
    fn new(err_kind: CmpErrorKind, err_pos: pos::Position) -> Self {
        Self {
            kind: err_kind,
            position: err_pos,
        }
    }

    fn format_with(&self, module_path: &str, message: String) -> String {
        format!(
            "{}{}: [{}] {}.",
            module_path,
            self.position,
            "CompileError".red().bold(),
            message
        )
    }

    pub fn emit_stderr(&self, module_path: &str, build_opt: &opt::BuildOption) {
        let message = match build_opt.language {
            opt::Language::JAPANESE => self.error_message_ja(),
            opt::Language::ENGLISH => self.error_message_en(),
        };

        eprintln!("{}", self.format_with(module_path, message));
    }

    pub fn error_message_en(&self) -> String {
        match &self.kind {
            CmpErrorKind::OUTOF64BITSINTRANGE(number_str) => format!(
                "'{}' is bigger than 64-bit signed integer's limit",
                number_str
            ),
        }
    }
    pub fn error_message_ja(&self) -> String {
        match &self.kind {
            CmpErrorKind::OUTOF64BITSINTRANGE(number_str) => format!(
                "数値リテラル '{}' は64bit符号付き整数で表現できる範囲を超えています",
                number_str
            ),
        }
    }

    pub fn out_of_64bit_sint_range(number_str: String, err_pos: pos::Position) -> Self {
        Self::new(CmpErrorKind::OUTOF64BITSINTRANGE(number_str), err_pos)
    }
}

#[derive(Clone)]
pub enum CmpErrorKind {
    /// 64bitより大きかった数字
    /// kind.0 -> 数値化できなかった数字列
    OUTOF64BITSINTRANGE(String),
}
