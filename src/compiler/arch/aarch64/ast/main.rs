use crate::compiler::arch::aarch64;
use crate::compiler::common::frontend::ast as high_ast;
use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    rc::Rc,
};

use aarch64::PeachiliType;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConstError {
    #[error("cannot evaluate constant => {name:?}")]
    CannotEvaluate { name: String },
}

/// 機械独立なASTから，aarch64依存なASTに作り変える
/// 基本的には同じ構造になるが，"Typed"にするという点で大きく異なる
pub fn ast_to_lower(
    common_root: &high_ast::ASTRoot,
    resolved_type_env: HashMap<String, PeachiliType>,
) -> Result<aarch64::Root, Box<dyn std::error::Error>> {
    let mut constants: HashMap<String, aarch64::Constant> = Default::default();
    let mut lower_functions: Vec<aarch64::Function> = Vec::with_capacity(common_root.decls.len());

    for common_tld in common_root.decls.iter() {
        match &common_tld.kind {
            high_ast::TopLevelDeclKind::PubConst {
                const_name,
                const_type: _,
                expr,
            } => {
                let const_value = evaluate_constant_expr(const_name, expr)?;
                constants.insert(const_name.to_string(), const_value);
            }
            high_ast::TopLevelDeclKind::Function {
                func_name,
                return_type,
                parameters,
                stmts,
            } => {
                let aarch64_fn = fn_to_lower(
                    &resolved_type_env,
                    &common_root.module_name,
                    (func_name, return_type, parameters, stmts),
                )?;
                lower_functions.push(aarch64_fn);
            }
            high_ast::TopLevelDeclKind::Import { module_name: _ } => {}
            high_ast::TopLevelDeclKind::PubType {
                type_name: _,
                to: _,
            } => {}
        }
    }

    Ok(aarch64::Root {
        functions: lower_functions,
        constants,
    })
}

/// high_ast::Function => aarch64::Function
fn fn_to_lower(
    resolved_type_env: &HashMap<String, PeachiliType>,
    ref_module_name: &String,
    f_attrs: (
        &String,
        &String,
        &HashMap<String, String>,
        &[high_ast::Stmt],
    ),
) -> Result<aarch64::Function, Box<dyn std::error::Error>> {
    let (fn_name, _return_type, params, stmts) = f_attrs;

    // 関数の返り値の型解決
    let mut local_variables: HashMap<String, aarch64::FrameObject> = Default::default();
    let mut fn_stack_size = 0;

    // 引数リストを解決する
    let lower_params = {
        let mut lower_params: HashMap<String, PeachiliType> = Default::default();
        for (param_name, param_type_name) in params.iter() {
            let param_type = match resolved_type_env.get(param_type_name) {
                Some(t) => t,
                None => resolved_type_env
                    .get(&format!("{}::{}", ref_module_name, param_type_name))
                    .unwrap(),
            };
            fn_stack_size += param_type.size();

            local_variables.insert(
                param_name.to_string(),
                aarch64::FrameObject {
                    stack_offset: fn_stack_size,
                    p_type: param_type.clone(),
                },
            );
            lower_params.insert(param_name.to_string(), param_type.clone());
        }
        lower_params
    };

    let mut lower_stmts = Vec::new();
    for stmt in stmts {
        lower_stmts.push(stmt_to_lower(stmt, &resolved_type_env, &mut fn_stack_size));
    }

    Ok(aarch64::Function {
        name: fn_name.to_string(),
        return_type: resolved_type_env.get(fn_name).unwrap().clone(),
        params: lower_params,
        local_variables,
        stack_size: fn_stack_size,
        stmts: lower_stmts,
    })
}

fn stmt_to_lower(
    stmt: &high_ast::Stmt,
    type_env: &HashMap<String, aarch64::PeachiliType>,
    stack_offset: &mut usize,
) -> aarch64::Statement {
    match &stmt.kind {
        high_ast::StmtKind::Expr { expr: expr_info } => aarch64::Statement::Expr {
            expr: expr_to_lower(&expr_info, type_env, stack_offset),
        },
        high_ast::StmtKind::Asm { insts } => aarch64::Statement::Asm {
            insts: insts.clone(),
        },
    }
}

fn expr_to_lower(
    expr: &high_ast::Expr,
    type_env: &HashMap<String, aarch64::PeachiliType>,
    stack_offset: &mut usize,
) -> aarch64::Expression {
    match &expr.kind {
        high_ast::ExprKind::Identifier { list } => match type_env.get(&list.join("::")) {
            Some(id_ty) => {
                *stack_offset += id_ty.size();
                aarch64::Expression::new(
                    aarch64::ExprKind::Identifier {
                        list: list.clone(),
                        stack_offset: *stack_offset,
                    },
                    id_ty.clone(),
                )
            }
            _ => unreachable!(),
        },
        high_ast::ExprKind::Integer { value } => aarch64::Expression::new(
            aarch64::ExprKind::Integer { value: *value },
            aarch64::PeachiliType::Int64,
        ),
        high_ast::ExprKind::UnsignedInteger { value } => aarch64::Expression::new(
            aarch64::ExprKind::UnsignedInteger { value: *value },
            aarch64::PeachiliType::Uint64,
        ),
        high_ast::ExprKind::Negative { child } => {
            let child_expr = expr_to_lower(&child.borrow(), type_env, stack_offset);
            let neg_ty = child_expr.ty.clone();
            aarch64::Expression::new(
                aarch64::ExprKind::Negative {
                    child: Rc::new(RefCell::new(child_expr)),
                },
                neg_ty,
            )
        }
        high_ast::ExprKind::StringLiteral { contents } => {
            let mut s = DefaultHasher::new();
            contents.hash(&mut s);
            aarch64::Expression::new(
                aarch64::ExprKind::StringLiteral {
                    contents: contents.to_string(),
                    id: s.finish(),
                },
                aarch64::PeachiliType::ConstStr,
            )
        }
        high_ast::ExprKind::Call { ident, params } => {
            let lower_ident = expr_to_lower(&ident.borrow(), type_env, stack_offset);
            let mut lower_params = Vec::new();
            for param in params.iter() {
                lower_params.push(expr_to_lower(param, type_env, stack_offset));
            }

            let call_ty = lower_ident.ty.clone();

            aarch64::Expression::new(
                aarch64::ExprKind::Call {
                    ident: match lower_ident.kind {
                        aarch64::ExprKind::Identifier {
                            list,
                            stack_offset: _,
                        } => list.join("::"),
                        _ => unreachable!(),
                    },
                    params: lower_params,
                },
                call_ty,
            )
        }
        high_ast::ExprKind::True => {
            aarch64::Expression::new(aarch64::ExprKind::True, aarch64::PeachiliType::Boolean)
        }
        high_ast::ExprKind::False => {
            aarch64::Expression::new(aarch64::ExprKind::False, aarch64::PeachiliType::Boolean)
        }
    }
}

fn evaluate_constant_expr(
    const_name: &String,
    const_expr: &high_ast::Expr,
) -> Result<aarch64::Constant, ConstError> {
    match const_expr.kind {
        // 64bit整数の範囲を超えていたらとりあえずエラー
        high_ast::ExprKind::Integer { value } => {
            if value > (std::i64::MAX as i128) {
                Err(ConstError::CannotEvaluate {
                    name: const_name.to_string(),
                })
            } else {
                Ok(aarch64::Constant::Integer(value as i64))
            }
        }
        high_ast::ExprKind::UnsignedInteger { value } => {
            if value > (std::u64::MAX as u128) {
                Err(ConstError::CannotEvaluate {
                    name: const_name.to_string(),
                })
            } else {
                Ok(aarch64::Constant::UnsignedInteger(value as u64))
            }
        }
        _ => Err(ConstError::CannotEvaluate {
            name: const_name.to_string(),
        }),
    }
}
