type Column = usize;
type Row = usize;

#[derive(PartialEq, Debug, Clone)]
pub struct Position {
    column: usize,
    row: usize,
}

#[allow(dead_code)]
impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { column: c, row: r }
    }

    pub fn get_pos(&self) -> (Row, Column) {
        (self.row, self.column)
    }

    pub fn add_col(&mut self, ex: usize) {
        self.column += ex;
    }

    pub fn add_row(&mut self, ex: usize) {
        self.row += ex;
    }

    pub fn set_col(&mut self, col: usize) {
        self.column = col;
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.column)
    }
}
