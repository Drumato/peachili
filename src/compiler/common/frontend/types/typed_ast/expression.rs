use crate::compiler::common::frontend::peachili_type;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Expression {
    pub kind: ExprKind,
    pub ty: peachili_type::PeachiliType,
}

impl Expression {
    pub fn new(k: ExprKind, ty: peachili_type::PeachiliType) -> Self {
        Self { kind: k, ty }
    }
}

#[derive(Debug)]
pub enum ExprKind {
    Integer {
        value: i128,
    },
    UnsignedInteger {
        value: u128,
    },
    Identifier {
        list: Vec<String>,
        stack_offset: usize,
    },
    Negative {
        child: Rc<RefCell<Expression>>,
    },
    Call {
        ident: String,
        params: Vec<Expression>,
    },
    StringLiteral {
        contents: String,
        id: u64,
    },
    True,
    False,
}
