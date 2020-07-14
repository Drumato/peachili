use crate::common::ast;
use crate::common::tld;
use crate::setup;
use std::collections::BTreeMap;

/// TLD解析のトップレベルルーチン
pub fn main(
    fn_arena: setup::FnArena,
    full_ast: &ast::ASTRoot,
) -> BTreeMap<String, tld::TopLevelDecl> {
    let mut tld_map: BTreeMap<String, tld::TopLevelDecl> = BTreeMap::new();

    for (alias_name, alias_type) in full_ast.alias.iter() {
        tld_map.insert(
            alias_name.to_string(),
            tld::TopLevelDecl::new(tld::TLDKind::ALIAS {
                src_type: alias_type.to_string(),
            }),
        );
    }

    for (type_name, struct_def) in full_ast.typedefs.iter() {
        tld_map.insert(
            type_name.to_string(),
            tld::TopLevelDecl::new(tld::TLDKind::STRUCT {
                members: struct_def.members.clone(),
            }),
        );
    }

    for fn_id in full_ast.funcs.iter() {
        let ast_function = fn_arena.lock().unwrap().get(*fn_id).unwrap().clone();

        let mut args: Vec<(String, String)> = Vec::new();

        for (name, t) in ast_function.args {
            args.push((name, t));
        }

        tld_map.insert(
            ast_function.name.to_string(),
            tld::TopLevelDecl::new(tld::TLDKind::FN {
                return_type: ast_function.return_type.clone(),
                args,
            }),
        );
    }

    tld_map
}
