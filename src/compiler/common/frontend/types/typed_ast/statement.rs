use crate::compiler::common::frontend::typed_ast;
pub enum Statement {
    Expr { expr: typed_ast::Expression },
    Asm { insts: Vec<String> },
    Nop,
}
