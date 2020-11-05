/// 式ノード
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    // pub position: position::Position,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum ExprKind {
    Integer { value: i128 },
    UnsignedInteger { value: u128 },
    Identifier { list: Vec<String> },
}
