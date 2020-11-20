use std::cell::RefCell;

/// 式ノード
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ExprInfo<'a> {
    pub kind: ExprKind<'a>,
    // pub position: position::Position,
}

pub type Expr<'a> = &'a ExprInfo<'a>;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum ExprKind<'a> {
    Negative { child: RefCell<Expr<'a>> },

    StringLiteral { contents: String },
    Integer { value: i128 },
    UnsignedInteger { value: u128 },
    Identifier { list: Vec<String> },
    True,
    False,
}
