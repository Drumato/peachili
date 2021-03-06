use crate::common::ast;
use crate::common::tld;
use std::collections::BTreeMap;

/// TLD解析のトップレベルルーチン
pub fn main(
    fn_arena: ast::FnArena,
    full_ast: &ast::ASTRoot,
) -> BTreeMap<String, tld::TopLevelDecl> {
    let mut tld_map: BTreeMap<String, tld::TopLevelDecl> = BTreeMap::new();

    for (alias_name, alias_type) in full_ast.alias.iter() {
        tld_map.insert(
            alias_name.to_string(),
            tld::TopLevelDecl::new_alias(alias_type),
        );
    }

    for (type_name, struct_def) in full_ast.typedefs.iter() {
        tld_map.insert(
            type_name.to_string(),
            tld::TopLevelDecl::new_struct_from_ast(struct_def.clone()),
        );
    }

    for (const_name, (const_type_name, const_expr)) in full_ast.constants.iter() {
        tld_map.insert(
            const_name.to_string(),
            tld::TopLevelDecl::new_const(const_type_name, const_expr.to_string()),
        );
    }

    for (enum_name, enum_decl) in full_ast.enum_decls.iter() {
        tld_map.insert(
            enum_name.to_string(),
            tld::TopLevelDecl::new_enum(enum_decl.clone()),
        );
    }

    for fn_id in full_ast.funcs.iter() {
        let ast_function = fn_arena.lock().unwrap().get(*fn_id).unwrap().clone();

        tld_map.insert(
            ast_function.name.to_string(),
            tld::TopLevelDecl::new_function_from_ast(ast_function.fn_type),
        );
    }

    tld_map
}
