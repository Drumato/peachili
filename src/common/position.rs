use std::fmt::{Display, Formatter, Result as FR};

/// ソースコード上の位置を保持する構造体
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct Position {
    // 行情報
    row: usize,
    // 列情報
    column: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self { row: 0, column: 0 }
    }
}

impl Position {
    pub fn new(row: usize, column: usize) -> Self {
        Self {
            row,
            column,
        }
    }
    /// 内部情報の取得
    pub fn get_info(&self) -> (usize, usize) {
        (self.row, self.column)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> FR {
        let (r, c) = self.get_info();
        write!(f, "({}, {})", r, c)
    }
}