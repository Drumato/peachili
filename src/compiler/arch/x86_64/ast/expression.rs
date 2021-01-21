use crate::compiler::arch::x86_64;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Expression {
    pub kind: ExprKind,
    pub ty: x86_64::PeachiliType,
}

impl Expression {
    pub fn new(k: ExprKind, ty: x86_64::PeachiliType) -> Self {
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
