use crate::common::{ast, error::CompileError, option, peachili_type::Type, tld};

use crate::common::compiler::analyzer::type_util;
use crate::common::error::TypeErrorKind;
use crate::common::peachili_type::TypeKind;
use std::collections::BTreeMap;

/// 型検査
pub fn type_check_main(
    fn_arena: ast::FnArena,
    tld_env: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &BTreeMap<String, BTreeMap<String, Type>>,
    ast_root: &ast::ASTRoot,
    target: option::Target,
) {
    // メイン関数が存在しなければエラー
    let mut main_func_exists = false;

    for fn_id in ast_root.funcs.iter() {
        if let Ok(fn_arena) = fn_arena.lock() {
            let function = fn_arena.get(*fn_id).unwrap();
            let func_name = function.name.clone();

            // メイン関数の場合，特別なチェックが必要
            if func_name == "main" {
                main_func_exists = true;
                if let Err(e) = type_check_main_fn(tld_env, type_env, function, target) {
                    e.output();
                    std::process::exit(1);
                }
            } else if let Err(e) = type_check_fn(tld_env, type_env, function, target) {
                e.output();
                std::process::exit(1);
            }
        }
    }

    // エントリポイントがなければエラー
    if !main_func_exists {
        CompileError::new(TypeErrorKind::MAINFUNCNOTFOUND, Default::default()).output();
        std::process::exit(1);
    }
}

/// メイン関数特有のチェック
fn type_check_main_fn(
    tld_env: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &BTreeMap<String, BTreeMap<String, Type>>,
    function: &ast::Function,
    target: option::Target,
) -> Result<(), CompileError<TypeErrorKind>> {
    // メイン関数では，以下のチェックが必要
    // - 引数が空になっているか
    // - 返り値の方がNoreturnになっているか

    if !function.args.is_empty() {
        return Err(CompileError::new(
            TypeErrorKind::MAINFUNCMUSTNOTHAVEANYARGUMENTS,
            function.pos,
        ));
    }

    if type_env.get("main").unwrap().get("main").unwrap().kind != TypeKind::NORETURN {
        return Err(CompileError::new(
            TypeErrorKind::MAINFUNCMUSTNOTRETURNANYVALUES,
            function.pos,
        ));
    }

    // 通常のチェックも同様に行う
    type_check_fn(tld_env, type_env, function, target)?;

    Ok(())
}

/// 関数に対するチェック
fn type_check_fn(
    tld_env: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &BTreeMap<String, BTreeMap<String, Type>>,
    function: &ast::Function,
    target: option::Target,
) -> Result<(), CompileError<TypeErrorKind>> {
    if let Ok(stmt_arena) = function.stmt_arena.lock() {
        for stmt_id in function.stmts.iter() {
            let stmt = stmt_arena.get(*stmt_id).unwrap();
            type_check_stmt(
                tld_env,
                type_env.get(&function.name).unwrap(),
                stmt,
                function.expr_arena.clone(),
                target,
            )?;
        }
    }

    Ok(())
}

/// 文に対するチェック
fn type_check_stmt(
    tld_env: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &BTreeMap<String, Type>,
    stmt: &ast::StatementNode,
    expr_arena: ast::ExprArena,
    target: option::Target,
) -> Result<(), CompileError<TypeErrorKind>> {
    match stmt.get_kind() {
        ast::StatementNodeKind::VARINIT {
            ident_name,
            type_name: _,
            expr,
        } => type_check_vardecl_stmt(tld_env, type_env, ident_name, *expr, expr_arena, target),
        ast::StatementNodeKind::RETURN { expr } => {
            type_check_return_stmt(tld_env, type_env, *expr, expr_arena, target)
        }
        _ => panic!("unimplemented type check with `{:?}`", stmt),
    }
}

// varinit文に関するチェック
fn type_check_vardecl_stmt(
    tld_env: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &BTreeMap<String, Type>,
    ident_name: &str,
    expr_id: ast::ExNodeId,
    expr_arena: ast::ExprArena,
    target: option::Target,
) -> Result<(), CompileError<TypeErrorKind>> {
    // varinit文に必要なチェック
    // - もちろんexpressionの型が検査できる
    // - 代入する識別子の型と式の型が一致している
    let _var_type = type_env.get(ident_name).unwrap();
    let _expr_type = type_check_expr(
        tld_env,
        type_env,
        expr_arena.clone(),
        expr_arena.lock().unwrap().get(expr_id).unwrap(),
        target,
    )?;

    Ok(())
}

// return文に関するチェック
fn type_check_return_stmt(
    tld_env: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &BTreeMap<String, Type>,
    expr_id: ast::ExNodeId,
    expr_arena: ast::ExprArena,
    target: option::Target,
) -> Result<(), CompileError<TypeErrorKind>> {
    // return文に必要なチェック
    // - もちろんexpressionの型が検査できる
    type_check_expr(
        tld_env,
        type_env,
        expr_arena.clone(),
        expr_arena.lock().unwrap().get(expr_id).unwrap(),
        target,
    )?;

    Ok(())
}

/// 式に対するチェック
fn type_check_expr(
    tld_env: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &BTreeMap<String, Type>,
    expr_arena: ast::ExprArena,
    expr: &ast::ExpressionNode,
    target: option::Target,
) -> Result<Type, CompileError<TypeErrorKind>> {
    match expr.get_kind() {
        ast::ExpressionNodeKind::INTEGER { value: _ } => Ok(Type::new_int64(
            type_util::resolve_type_size(tld_env, type_util::ForCalcTypeSize::INT64, target),
        )),
        ast::ExpressionNodeKind::UINTEGER { value: _ } => Ok(Type::new_uint64(
            type_util::resolve_type_size(tld_env, type_util::ForCalcTypeSize::UINT64, target),
        )),
        ast::ExpressionNodeKind::IDENTIFIER { names } => {
            let full_path = names.join("::");
            Ok(type_env.get(&full_path).unwrap().clone())
        }
        ast::ExpressionNodeKind::BOOLEAN { truth: _ } => Ok(Type::new_boolean(
            type_util::resolve_type_size(tld_env, type_util::ForCalcTypeSize::BOOLEAN, target),
        )),
        ast::ExpressionNodeKind::STRING { contents: _ } => Ok(Type::new_const_str(
            type_util::resolve_type_size(tld_env, type_util::ForCalcTypeSize::CONSTSTR, target),
        )),
        ast::ExpressionNodeKind::MEMBER {
            id: st_id,
            member: member_id,
        } => {
            let struct_node = expr_arena.lock().unwrap().get(*st_id).unwrap().clone();
            let member_node = expr_arena.lock().unwrap().get(*member_id).unwrap().clone();
            type_check_member_expression(tld_env, type_env, struct_node, member_node, target)
        }
        _ => panic!("unimplemented type check with `{:?}`", expr),
    }
}

fn type_check_member_expression(
    _tld_env: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &BTreeMap<String, Type>,
    struct_node: ast::ExpressionNode,
    value_node: ast::ExpressionNode,
    _target: option::Target,
) -> Result<Type, CompileError<TypeErrorKind>> {
    // メンバ式でチェックすること
    // - DOTの前後ノードが変数であるか
    // - DOT前のノードが構造体型であるか
    // - メンバ名が構造体に存在するか
    if !struct_node.is_identifier() {
        let err_pos = struct_node.get_pos();
        return Err(CompileError::new(
            TypeErrorKind::CANNOTACCESSMEMBERWITHNOTANIDENTIFIER { struct_node },
            err_pos,
        ));
    }

    if !value_node.is_identifier() {
        let err_pos = value_node.get_pos();
        return Err(CompileError::new(
            TypeErrorKind::MEMBERNAMEMUSTBEANIDENTIFIER {
                member_node: value_node,
            },
            err_pos,
        ));
    }

    // 型環境から，ドット前のノードの型を持ってくる
    match type_env.get(&struct_node.copy_names().join("::")) {
        Some(node_type) => {
            // 構造体型でなければエラー
            if !node_type.is_struct() {
                let err_pos = struct_node.get_pos();
                return Err(CompileError::new(
                    TypeErrorKind::CANNOTACCESSMEMBERWITHNOTASTRUCT { struct_node },
                    err_pos,
                ));
            }

            let members = node_type.get_members();

            // メンバが存在するかチェック
            match members.get(&value_node.copy_names().join("::")) {
                Some((member_type, _member_offset)) => Ok(*member_type.clone()),
                None => {
                    let err_pos = value_node.get_pos();
                    Err(CompileError::new(
                        TypeErrorKind::UNDEFINEDSUCHAMEMBER {
                            member_node: value_node,
                        },
                        err_pos,
                    ))
                }
            }
        }
        None => unimplemented!(),
    }
}

#[cfg(test)]
mod type_check_tests {
    use super::*;
    use crate::common::token::TokenKind;
    use id_arena::Arena;
    use std::sync::{Arc, Mutex};

    #[test]
    fn type_check_main_fn_with_invalid_return_type_test() {
        let invalid_return_func = new_func("main".to_string(), Default::default());
        let tld_env = new_tld();

        let env = type_env();

        let actual =
            type_check_main_fn(&tld_env, &env, &invalid_return_func, option::Target::X86_64);
        assert!(actual.is_err());

        if let Err(e) = actual {
            assert_eq!(&TypeErrorKind::MAINFUNCMUSTNOTRETURNANYVALUES, e.get_kind());
        }
    }

    #[test]
    fn type_check_main_fn_with_invalid_args_test() {
        let invalid_args_func = new_func("main".to_string(), {
            let mut args = BTreeMap::new();
            args.insert("foo".to_string(), "Int64".to_string());
            args
        });
        let tld_env = new_tld();
        let env = type_env();

        let actual = type_check_main_fn(&tld_env, &env, &invalid_args_func, option::Target::X86_64);
        assert!(actual.is_err());

        if let Err(e) = actual {
            assert_eq!(
                &TypeErrorKind::MAINFUNCMUSTNOTHAVEANYARGUMENTS,
                e.get_kind()
            );
        }
    }

    #[test]
    fn invalid_member_access_test() {
        let (_fn_arena, expr_arena) = new_allocators();
        let tld_env = new_tld();
        let env = new_func_env();

        // `3.foo`
        let member_ex = new_member_node(
            expr_arena.clone(),
            ast::ExpressionNode::new_integer(3, Default::default()),
            ast::ExpressionNode::new_identifier(vec!["foo".to_string()], Default::default()),
        );
        let member_type = type_check_expr(
            &tld_env,
            &env,
            expr_arena.clone(),
            &member_ex,
            option::Target::X86_64,
        );
        type_check_expr_error_test(
            member_type,
            TypeErrorKind::CANNOTACCESSMEMBERWITHNOTANIDENTIFIER {
                struct_node: ast::ExpressionNode::new_integer(3, Default::default()),
            },
        );

        // `foo.3`
        let member_ex = new_member_node(
            expr_arena.clone(),
            ast::ExpressionNode::new_identifier(vec!["foo".to_string()], Default::default()),
            ast::ExpressionNode::new_integer(3, Default::default()),
        );
        let member_type = type_check_expr(
            &tld_env,
            &env,
            expr_arena.clone(),
            &member_ex,
            option::Target::X86_64,
        );
        type_check_expr_error_test(
            member_type,
            TypeErrorKind::MEMBERNAMEMUSTBEANIDENTIFIER {
                member_node: ast::ExpressionNode::new_integer(3, Default::default()),
            },
        );

        // `x.foo`
        let member_ex = new_member_node(
            expr_arena.clone(),
            ast::ExpressionNode::new_identifier(vec!["x".to_string()], Default::default()),
            ast::ExpressionNode::new_identifier(vec!["foo".to_string()], Default::default()),
        );
        let member_type = type_check_expr(
            &tld_env,
            &env,
            expr_arena.clone(),
            &member_ex,
            option::Target::X86_64,
        );
        type_check_expr_error_test(
            member_type,
            TypeErrorKind::CANNOTACCESSMEMBERWITHNOTASTRUCT {
                struct_node: ast::ExpressionNode::new_identifier(
                    vec!["x".to_string()],
                    Default::default(),
                ),
            },
        );

        // `st.undefined`
        let member_ex = new_member_node(
            expr_arena.clone(),
            ast::ExpressionNode::new_identifier(vec!["st".to_string()], Default::default()),
            ast::ExpressionNode::new_identifier(vec!["undefined".to_string()], Default::default()),
        );
        let member_type = type_check_expr(
            &tld_env,
            &env,
            expr_arena.clone(),
            &member_ex,
            option::Target::X86_64,
        );
        type_check_expr_error_test(
            member_type,
            TypeErrorKind::UNDEFINEDSUCHAMEMBER {
                member_node: ast::ExpressionNode::new_identifier(
                    vec!["undefined".to_string()],
                    Default::default(),
                ),
            },
        );
    }

    fn type_check_expr_error_test(
        actual: Result<Type, CompileError<TypeErrorKind>>,
        expected_error: TypeErrorKind,
    ) {
        assert!(actual.is_err());

        if let Err(e) = actual {
            assert_eq!(&expected_error, e.get_kind());
        }
    }

    #[test]
    fn type_check_expr_test() {
        let (_fn_arena, expr_arena) = new_allocators();
        let tld_env = new_tld();
        let env = new_func_env();

        // integer-literal
        let int_ex = ast::ExpressionNode::new_integer(30, Default::default());
        let int_type = type_check_expr(
            &tld_env,
            &env,
            expr_arena.clone(),
            &int_ex,
            option::Target::X86_64,
        );
        assert!(int_type.is_ok());
        assert_eq!(Type::new_int64(8), int_type.unwrap());

        // identifier-literal
        let uint_ex = ast::ExpressionNode::new_uinteger(30, Default::default());
        let uint_type = type_check_expr(
            &tld_env,
            &env,
            expr_arena.clone(),
            &uint_ex,
            option::Target::X86_64,
        );
        assert!(uint_type.is_ok());
        assert_eq!(Type::new_uint64(8), uint_type.unwrap());

        // boolean-literal
        let bool_ex = ast::ExpressionNode::new_boolean(true, Default::default());
        let bool_type = type_check_expr(
            &tld_env,
            &env,
            expr_arena.clone(),
            &bool_ex,
            option::Target::X86_64,
        );
        assert!(bool_type.is_ok());
        assert_eq!(Type::new_boolean(8), bool_type.unwrap());

        // identifier
        let ident_ex =
            ast::ExpressionNode::new_identifier(vec!["x".to_string()], Default::default());
        let ident_type = type_check_expr(
            &tld_env,
            &env,
            expr_arena.clone(),
            &ident_ex,
            option::Target::X86_64,
        );
        assert!(ident_type.is_ok());
        assert_eq!(Type::new_int64(8), ident_type.unwrap());

        // string-literal
        let strlit_ex =
            ast::ExpressionNode::new_string_literal("Drumato".to_string(), Default::default());
        let strlit_type = type_check_expr(
            &tld_env,
            &env,
            expr_arena.clone(),
            &strlit_ex,
            option::Target::X86_64,
        );
        assert!(strlit_type.is_ok());
        assert_eq!(Type::new_const_str(8), strlit_type.unwrap());

        // member_expr
        let member_ex = new_member_node(
            expr_arena.clone(),
            ast::ExpressionNode::new_identifier(vec!["st".to_string()], Default::default()),
            ast::ExpressionNode::new_identifier(vec!["foo".to_string()], Default::default()),
        );
        let member_type = type_check_expr(
            &tld_env,
            &env,
            expr_arena.clone(),
            &member_ex,
            option::Target::X86_64,
        );
        assert!(member_type.is_ok());
        assert_eq!(Type::new_int64(8), member_type.unwrap());
    }

    fn new_member_node(
        expr_arena: ast::ExprArena,
        st_node: ast::ExpressionNode,
        mem_node: ast::ExpressionNode,
    ) -> ast::ExpressionNode {
        let st_id = expr_arena.lock().unwrap().alloc(st_node);
        let mem_id = expr_arena.lock().unwrap().alloc(mem_node);
        ast::ExpressionNode::new_postfix_op(&TokenKind::DOT, st_id, mem_id, Default::default())
    }

    fn new_func(name: String, args: BTreeMap<String, String>) -> ast::Function {
        ast::Function {
            name,
            stmts: vec![],
            return_type: "".to_string(),
            args,
            pos: Default::default(),
            module_name: "".to_string(),
            stmt_arena: Arc::new(Mutex::new(Default::default())),
            expr_arena: Arc::new(Mutex::new(Default::default())),
        }
    }

    fn type_env() -> BTreeMap<String, BTreeMap<String, Type>> {
        let mut e = BTreeMap::new();

        e.insert("main".to_string(), new_func_env());

        e
    }

    fn new_func_env() -> BTreeMap<String, Type> {
        let mut func_env = BTreeMap::new();
        // invalidなmain関数の型
        func_env.insert("main".to_string(), Type::new_int64(8));

        // なんてことない変数
        func_env.insert("x".to_string(), Type::new_int64(8));

        // 構造体変数
        func_env.insert(
            "st".to_string(),
            Type::new_struct(
                {
                    let mut members = BTreeMap::new();
                    members.insert("foo".to_string(), (Box::new(Type::new_int64(8)), 0));

                    members.insert("bar".to_string(), (Box::new(Type::new_int64(8)), 8));
                    members
                },
                16,
            ),
        );

        func_env
    }

    fn new_allocators() -> (ast::FnArena, ast::ExprArena) {
        (
            Arc::new(Mutex::new(Arena::new())),
            Arc::new(Mutex::new(Arena::new())),
        )
    }

    fn new_tld() -> BTreeMap<String, tld::TopLevelDecl> {
        let mut m = BTreeMap::new();

        m.insert(
            "T1".to_string(),
            tld::TopLevelDecl::new(tld::TLDKind::ALIAS {
                src_type: "Int64".to_string(),
            }),
        );
        m.insert(
            "S1".to_string(),
            tld::TopLevelDecl::new(tld::TLDKind::STRUCT {
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
