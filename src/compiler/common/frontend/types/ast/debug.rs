use super::{ASTRoot, StmtInfo, StmtKind, TopLevelDecl, TopLevelDeclKind};
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
            stmts,
        } => {
            eprintln!("Function {}() {} {{", func_name, return_type);
            for stmt in stmts.iter() {
                dump_stmt(stmt);
            }
            eprintln!("}}");
        }
        TopLevelDeclKind::Import { module_name } => {
            eprintln!("Import {};", module_name);
        }
    }
}

fn dump_stmt(stmt: &StmtInfo) {
    match &stmt.kind {
        StmtKind::Expr { expr: _ } => {
            eprintln!("    ExpressionStatement");
        }
    }
}
