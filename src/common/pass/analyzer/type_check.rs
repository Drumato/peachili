use crate::common::{ast, error::CompileError, option, peachili_type::Type, tld};

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
        CompileError::new(TypeErrorKind::NotFoundMainFunction, Default::default()).output();
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

    if !function.get_parameters().is_empty() {
        return Err(CompileError::new(
            TypeErrorKind::MAINFUNCMUSTNOTHAVEANYARGUMENTS,
            function.pos,
        ));
    }

    if type_env.get("main").unwrap().get("main").unwrap().kind != TypeKind::NORETURN {
        return Err(CompileError::new(
            TypeErrorKind::MainFunctionMustNotReturnAnyValues,
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
        ast::ExpressionNodeKind::INTEGER { value: _ } => Ok(Type::new_int64(target)),
        ast::ExpressionNodeKind::UINTEGER { value: _ } => Ok(Type::new_uint64(target)),
        ast::ExpressionNodeKind::IDENTIFIER { names } => {
            let full_path = names.join("::");
            Ok(type_env.get(&full_path).unwrap().clone())
        }
        ast::ExpressionNodeKind::BOOLEAN { truth: _ } => Ok(Type::new_boolean(target)),
        ast::ExpressionNodeKind::STRING { contents: _ } => Ok(Type::new_const_str(target)),
        ast::ExpressionNodeKind::MEMBER { id: st_id, member } => {
            let struct_node = expr_arena.lock().unwrap().get(*st_id).unwrap().clone();
            type_check_member_expression(tld_env, type_env, struct_node, member, target)
        }
        _ => panic!("unimplemented type check with `{:?}`", expr),
    }
}

fn type_check_member_expression(
    _tld_env: &BTreeMap<String, tld::TopLevelDecl>,
    type_env: &BTreeMap<String, Type>,
    struct_node: ast::ExpressionNode,
    member: &str,
    _target: option::Target,
) -> Result<Type, CompileError<TypeErrorKind>> {
    // メンバ式でチェックすること
    // - DOTの前後ノードが変数であるか
    // - DOT前のノードが構造体型であるか
    // - メンバ名が構造体に存在するか
    if !struct_node.is_identifier() {
        let err_pos = struct_node.get_pos();
        return Err(CompileError::new(
            TypeErrorKind::CannotAccessMemberWithNotAnIdentifier { struct_node },
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
                    TypeErrorKind::CannotAccessMemberWIthNotAStruct { struct_node },
                    err_pos,
                ));
            }

            let members = node_type.get_members();

            // メンバが存在するかチェック
            match members.get(member) {
                Some((member_type, _member_offset)) => Ok(*member_type.clone()),
                None => {
                    let err_pos = struct_node.get_pos();
                    Err(CompileError::new(
                        TypeErrorKind::UndefinedSuchAMember {
                            member: member.to_string(),
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
mod type_check_tests {}
