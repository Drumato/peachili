use crate::compiler::arch::x86_64::ast;

pub enum Statement {
    Expr { expr: ast::Expression },
    Asm { insts: Vec<String> },
}
