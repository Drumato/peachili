use super::{ASTRoot, Stmt, StmtKind, TopLevelDecl, TopLevelDeclKind};
pub fn dump_ast_root(ast_root: &ASTRoot) {
    for decl in ast_root.decls.iter() {
        dump_decl(decl);
    }
}

fn dump_decl(decl: &TopLevelDecl) {
    match &decl.kind {
        TopLevelDeclKind::Function {
            func_name,
            return_type,
            parameters: _,
            stmts,
        } => {
            eprintln!("Function {}() {} {{", func_name, return_type);
            for stmt in stmts.iter() {
                dump_stmt(stmt);
            }
            eprintln!("}}");
        }
        TopLevelDeclKind::PubType { type_name, to } => {
            eprintln!("PubType {} = {};", type_name, to);
        }
        TopLevelDeclKind::PubConst {
            const_name,
            const_type,
            expr: _,
        } => {
            eprintln!("PubConst {}: {};", const_name, const_type);
        }
        TopLevelDeclKind::Import { module_name } => {
            eprintln!("Import {};", module_name);
        }
    }
}

fn dump_stmt(stmt: &Stmt) {
    match &stmt.kind {
        StmtKind::HalfOpenCountup { .. } => {
            eprintln!("    HalfOpenCountupStatement",);
        }
        StmtKind::ClosedCountup { .. } => {
            eprintln!("    ClosedCountupStatement",);
        }
        StmtKind::Block { .. } => {
            eprintln!("    BlockStatement",);
        }
        StmtKind::Declare {
            var_name,
            type_name,
        } => {
            eprintln!(
                "    DeclareStatement(var: {}, type: {})",
                var_name,
                type_name.join("::")
            );
        }
        StmtKind::Expr { expr: _ } => {
            eprintln!("    ExpressionStatement");
        }
        StmtKind::Asm { insts: _ } => {
            eprintln!("    AsmStatement");
        }
    }
}
