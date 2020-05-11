extern crate colored;

use colored::*;

use crate::common::position as pos;

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

    pub fn emit_stderr(&self, module_path: &str) {
        let message = match &self.kind {
            CmpErrorKind::OUTOF64BITSINTRANGE(number_str) => format!(
                "{} is bigger than 64-bit signed integer's limit",
                number_str
            ),
        };

        eprintln!("{}", self.format_with(module_path, message));
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
