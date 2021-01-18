use std::cell::RefCell;
use std::rc::Rc;

/// 式ノード
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    // pub position: position::Position,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum ExprKind {
    Negative {
        child: Rc<RefCell<Expr>>,
    },

    StringLiteral {
        contents: String,
    },
    Integer {
        value: i128,
    },
    UnsignedInteger {
        value: u128,
    },
    Identifier {
        list: Vec<String>,
    },
    Call {
        ident: Rc<RefCell<Expr>>,
        params: Vec<Expr>,
    },
    True,
    False,
}
