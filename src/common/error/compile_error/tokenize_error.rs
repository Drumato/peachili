use fmt::Formatter;
use std::fmt;

use crate::common::error::CompileErrorKind;

/// Tokenizerが発行するエラーの種類を列挙
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum TokenizeErrorKind {
    /// 整数トークンが許容範囲外であった
    INTEGERLITERALOUTOFRANGE(String),

    /// これ以上トークナイズできない
    SOURCEISEMPTY,
}

impl CompileErrorKind for TokenizeErrorKind {
    fn category(&self) -> &'static str {
        "TokenizeError"
    }
}

impl fmt::Display for TokenizeErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            TokenizeErrorKind::INTEGERLITERALOUTOFRANGE(number) => {
                format!("an int-literal `{}` out of range 64bit", number)
            }
            TokenizeErrorKind::SOURCEISEMPTY => "source is empty".to_string(),
        };

        write!(f, "{}", s)
    }
}
