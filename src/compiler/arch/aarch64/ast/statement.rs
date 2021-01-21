use crate::compiler::arch::aarch64::ast;

pub enum Statement {
    Expr { expr: ast::Expression },
    Asm { insts: Vec<String> },
}
