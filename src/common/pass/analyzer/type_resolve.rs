use crate::common::{ast, error::CompileError, option, peachili_type::Type, tld};

use crate::common::error::TypeErrorKind;
use std::collections::BTreeMap;

/// 型情報の収集．
pub fn type_resolve_main(
    fn_arena: ast::FnArena,
    tld_map: &BTreeMap<String, tld::TopLevelDecl>,
    ast_root: &ast::ASTRoot,
    target: option::Target,
) -> BTreeMap<String, BTreeMap<String, Type>> {
    let mut type_env = BTreeMap::new();
    type_env.insert("global".to_string(), BTreeMap::new());

    // 先に型定義，エイリアスをすべて処理してしまう
    for (alias_name, alias_type_str) in ast_root.alias.iter() {
        let alias_type = resolve_type_string(tld_map, alias_type_str.to_string(), target);

        if let Err(e) = &alias_type {
            e.output();
            std::process::exit(1);
        }

        // グローバル領域に書き込んでおく
        if let Some(global_env) = type_env.get_mut("global") {
            global_env.insert(alias_name.to_string(), alias_type.unwrap());
        }
    }

    // 関数列を操作し，関数内の識別子に型をつけていく．
    for fn_id in ast_root.funcs.iter() {
        let mut func_env = BTreeMap::new();
        if let Ok(arena) = fn_arena.lock() {
            let function = arena.get(*fn_id).unwrap();

            // 関数自体の型格納
            let function_ret_type =
                resolve_type_string(tld_map, function.copy_return_type(), target);
            if let Err(e) = function_ret_type {
                e.output();
                std::process::exit(1);
            }

            func_env.insert(
                function.full_path(),
                Type::new_function(function_ret_type.unwrap()),
            );

            if let Err(e) = add_auto_var_to_env(tld_map, &mut type_env, function, target) {
                e.output();
                std::process::exit(1);
            }

            // 自動変数の型格納
            for (arg_name, arg_type_str) in function.get_parameters().iter() {
                let var_type = resolve_type_string(tld_map, arg_type_str.clone(), target);
                if let Err(e) = var_type {
                    e.output();
                    std::process::exit(1);
                }

                func_env.insert(arg_name.clone(), var_type.unwrap());
            }

            for stmt_id in function.stmts.iter() {
                if let Ok(stmt_arena) = function.stmt_arena.lock() {
                    let stmt = stmt_arena.get(*stmt_id).unwrap();

                    match stmt.get_kind() {
                        ast::StatementNodeKind::DECLARE {
                            ident_name,
                            type_name,
                        } => {
                            let var_type = resolve_type_string(tld_map, type_name.clone(), target);
                            if let Err(e) = var_type {
                                e.output();
                                std::process::exit(1);
                            }

                            func_env.insert(ident_name.clone(), var_type.unwrap());
                        }
                        ast::StatementNodeKind::VARINIT {
                            ident_name,
                            type_name,
                            expr: _,
                        } => {
                            let var_type = resolve_type_string(tld_map, type_name.clone(), target);
                            if let Err(e) = var_type {
                                e.output();
                                std::process::exit(1);
                            }

                            func_env.insert(ident_name.clone(), var_type.unwrap());
                        }
                        ast::StatementNodeKind::CONST {
                            ident_name,
                            type_name,
                            expr: _,
                        } => {
                            let var_type = resolve_type_string(tld_map, type_name.clone(), target);
                            if let Err(e) = var_type {
                                e.output();
                                std::process::exit(1);
                            }

                            func_env.insert(ident_name.clone(), var_type.unwrap());
                        }
                        _ => {}
                    }
                }
            }

            type_env.insert(function.full_path(), func_env);
        }
    }

    type_env
}

fn add_auto_var_to_env(
    tld_map: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &mut BTreeMap<String, BTreeMap<String, Type>>,
    function: &ast::Function,
    target: option::Target,
) -> Result<(), CompileError<TypeErrorKind>> {
    let func_name = function.name.to_string();
    type_env.insert(func_name.to_string(), BTreeMap::new());

    // 引数のデータ格納
    for (arg_name, arg_type_str) in function.get_parameters().iter() {
        if let Some(locals) = type_env.get_mut(&func_name) {
            let arg_type = resolve_type_string(tld_map, arg_type_str.to_string(), target)?;
            locals.insert(arg_name.to_string(), arg_type);
        }
    }

    // 変数宣言系のデータ格納
    for stmt_id in function.stmts.iter() {
        if let Ok(arena) = function.stmt_arena.lock() {
            let stmt = arena.get(*stmt_id).unwrap();

            match stmt.get_kind() {
                ast::StatementNodeKind::DECLARE {
                    ident_name,
                    type_name,
                } => {
                    let var_type = resolve_type_string(tld_map, type_name.to_string(), target)?;

                    if let Some(locals) = type_env.get_mut(&func_name) {
                        locals.insert(ident_name.to_string(), var_type);
                    }
                }
                ast::StatementNodeKind::CONST {
                    ident_name,
                    type_name,
                    expr: _,
                } => {
                    let var_type = resolve_type_string(tld_map, type_name.to_string(), target)?;

                    if let Some(locals) = type_env.get_mut(&func_name) {
                        locals.insert(ident_name.to_string(), var_type);
                    }
                }
                ast::StatementNodeKind::VARINIT {
                    ident_name,
                    type_name,
                    expr: _,
                } => {
                    let var_type = resolve_type_string(tld_map, type_name.to_string(), target)?;

                    if let Some(locals) = type_env.get_mut(&func_name) {
                        locals.insert(ident_name.to_string(), var_type);
                    }
                }

                _ => {}
            }
        }
    }

    for stmt_id in function.stmts.iter() {
        if let Ok(arena) = function.stmt_arena.lock() {
            let stmt = arena.get(*stmt_id).unwrap();

            match stmt.get_kind() {
                ast::StatementNodeKind::DECLARE {
                    ident_name,
                    type_name,
                } => {
                    let var_type = resolve_type_string(tld_map, type_name.to_string(), target)?;

                    if let Some(locals) = type_env.get_mut(&func_name) {
                        locals.insert(ident_name.to_string(), var_type);
                    }
                }
                ast::StatementNodeKind::CONST {
                    ident_name,
                    type_name,
                    expr: _,
                } => {
                    let var_type = resolve_type_string(tld_map, type_name.to_string(), target)?;

                    if let Some(locals) = type_env.get_mut(&func_name) {
                        locals.insert(ident_name.to_string(), var_type);
                    }
                }
                ast::StatementNodeKind::VARINIT {
                    ident_name,
                    type_name,
                    expr: _,
                } => {
                    let var_type = resolve_type_string(tld_map, type_name.to_string(), target)?;

                    if let Some(locals) = type_env.get_mut(&func_name) {
                        locals.insert(ident_name.to_string(), var_type);
                    }
                }

                _ => {}
            }
        }
    }

    Ok(())
}

/// type_string -> `*` type_string | primitive_types
fn resolve_type_string(
    tld_map: &BTreeMap<String, tld::TopLevelDecl>,
    type_name_str: String,
    target: option::Target,
) -> Result<Type, CompileError<TypeErrorKind>> {
    if type_name_str.starts_with('*') {
        let pointer_to = resolve_type_string(tld_map, type_name_str[1..].to_string(), target)?;
        return Ok(Type::new_pointer(pointer_to, target));
    }

    match type_name_str.as_str() {
        "Int64" => Ok(Type::new_int64(target)),
        "Uint64" => Ok(Type::new_uint64(target)),
        "ConstStr" => Ok(Type::new_const_str(target)),
        "Noreturn" => Ok(Type::new_noreturn()),
        _ => {
            // TopLevelDeclから探す，なかったらいよいよエラー
            if let Some(tld_entry) = tld_map.get(&type_name_str) {
                return resolve_type_from_tld(
                    type_name_str.to_string(),
                    tld_map,
                    tld_entry,
                    target,
                );
            }

            Err(CompileError::new(
                TypeErrorKind::CANNOTRESOLVE {
                    type_name: type_name_str,
                },
                Default::default(),
            ))
        }
    }
}

/// TopLevelDecl領域を探索して，対象の型を返す
fn resolve_type_from_tld(
    type_name_str: String,
    tld_map: &BTreeMap<String, tld::TopLevelDecl>,
    entry: &tld::TopLevelDecl,
    target: option::Target,
) -> Result<Type, CompileError<TypeErrorKind>> {
    match &entry.kind {
        tld::TLDKind::ALIAS { src_type } => {
            resolve_type_string(tld_map, src_type.to_string(), target)
        }
        tld::TLDKind::STRUCT { members } => {
            let mut member_types = BTreeMap::new();
            let mut total_size = 0;

            for (member_n, member_t) in members {
                let member_type = resolve_type_string(tld_map, member_t.to_string(), target)?;

                let member_offset = total_size;
                total_size += member_type.size;

                member_types.insert(member_n.to_string(), (Box::new(member_type), member_offset));
            }

            Ok(Type::new_struct(member_types, total_size))
        }
        // 関数名だったときは何もしない．
        tld::TLDKind::FN {
            return_type: _,
            args: _,
        } => Err(CompileError::new(
            TypeErrorKind::GOTFUNCTIONNAMEASTYPE {
                func_name: type_name_str,
            },
            Default::default(),
        )),
    }
}

#[cfg(test)]
mod analyze_tests {
    use super::*;
    use crate::common::option::Target;
    use crate::common::tld::{TLDKind, TopLevelDecl};

    #[test]
    fn resolve_type_string_in_x64_test() {
        let m = new_tld();

        check_types(Type::new_noreturn(), &m, "Noreturn", option::Target::X86_64);
        check_types(
            Type::new_int64(Target::X86_64),
            &m,
            "Int64",
            option::Target::X86_64,
        );
        check_types(
            Type::new_uint64(Target::X86_64),
            &m,
            "Uint64",
            option::Target::X86_64,
        );
        check_types(
            Type::new_pointer(Type::new_int64(Target::X86_64), Target::X86_64),
            &m,
            "*Int64",
            option::Target::X86_64,
        );

        check_types(
            Type::new_int64(Target::X86_64),
            &m,
            "T1",
            option::Target::X86_64,
        );
        check_types(
            Type::new_struct(
                {
                    let mut mm = BTreeMap::new();
                    mm.insert(
                        "m1".to_string(),
                        (Box::new(Type::new_int64(Target::X86_64)), 0),
                    );
                    mm.insert(
                        "m2".to_string(),
                        (Box::new(Type::new_uint64(Target::X86_64)), 8),
                    );
                    mm
                },
                16,
            ),
            &m,
            "S1",
            option::Target::X86_64,
        );

        assert!(resolve_type_string(&m, "T2".to_string(), option::Target::X86_64).is_err());
    }

    fn check_types(
        expected: Type,
        m: &BTreeMap<String, tld::TopLevelDecl>,
        s: &str,
        t: option::Target,
    ) {
        let actual = resolve_type_string(m, s.to_string(), t);
        assert!(actual.is_ok());

        assert_eq!(expected, actual.unwrap());
    }

    fn new_tld() -> BTreeMap<String, tld::TopLevelDecl> {
        let mut m = BTreeMap::new();

        m.insert(
            "T1".to_string(),
            TopLevelDecl::new(TLDKind::ALIAS {
                src_type: "Int64".to_string(),
            }),
        );
        m.insert(
            "S1".to_string(),
            TopLevelDecl::new(TLDKind::STRUCT {
                members: {
                    let mut mm = BTreeMap::new();
                    mm.insert("m1".to_string(), "Int64".to_string());
                    mm.insert("m2".to_string(), "Uint64".to_string());
                    mm
                },
            }),
        );

        m
    }
}
