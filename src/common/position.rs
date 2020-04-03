type Column = usize;
type Row = usize;

#[derive(PartialEq, Debug, Clone)]
pub struct Position {
    column: usize,
    row: usize,
}

impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { column: c, row: r }
    }

    pub fn get_pos(&self) -> (Row, Column) {
        (self.row, self.column)
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.column)
    }
}
