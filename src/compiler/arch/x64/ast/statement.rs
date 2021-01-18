use crate::compiler::arch::x64::ast;

pub enum Statement {
    Expr { expr: ast::Expression },
    Asm { insts: Vec<String> },
}
