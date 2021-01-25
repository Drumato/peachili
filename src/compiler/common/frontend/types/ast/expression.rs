use std::cell::RefCell;
use std::rc::Rc;

/// 式ノード
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    // pub position: position::Position,
}

impl Expr {
    pub fn new_edge(n: Self) -> Edge {
        Rc::new(RefCell::new(n))
    }
}

type Edge = Rc<RefCell<Expr>>;
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum ExprKind {
    /// `1 + 1`
    Addition {
        lhs: Edge,
        rhs: Edge,
    },
    /// `200 - 100`
    Subtraction {
        lhs: Edge,
        rhs: Edge,
    },
    /// `10 * 20`
    Multiplication {
        lhs: Edge,
        rhs: Edge,
    },
    /// `200 / 2`
    Division {
        lhs: Edge,
        rhs: Edge,
    },
    Negative {
        child: Edge,
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
