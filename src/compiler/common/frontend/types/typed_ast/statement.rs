use crate::compiler::common::frontend::typed_ast;

pub enum Statement {
    HalfOpenCountup {
        block: Vec<Statement>,
        block_id: usize,
        scope: typed_ast::Scope,
        var_name: String,
        var_stack_offset: usize,
        from: typed_ast::Expression,
        lessthan: typed_ast::Expression,
    },
    ClosedCountup {
        block: Vec<Statement>,
        block_id: usize,
        scope: typed_ast::Scope,
        var_name: String,
        var_stack_offset: usize,
        from: typed_ast::Expression,
        to: typed_ast::Expression,
    },
    Expr {
        expr: typed_ast::Expression,
    },
    Asm {
        insts: Vec<String>,
    },
    Block {
        block_id: usize,
        stmts: Vec<Statement>,
        scope: typed_ast::Scope,
    },
    Nop,
}
