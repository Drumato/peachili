use crate::compiler::arch::x64;
use crate::compiler::common::frontend::ast as high_ast;
use fxhash::FxHashMap;
use std::collections::HashMap;

use thiserror::Error;
use x64::PeachiliType;

#[derive(Error, Debug)]
pub enum TypeResolveError {
    #[error("cannot resolve type of '{name:?}'")]
    CannotResolve { name: String },
    #[error("cannot evaluate '{name:?}' as a const now")]
    CannotEvaluateConst { name: String },
}
pub fn ast_to_lower(
    common_root: &high_ast::ASTRoot,
    raw_type_env: FxHashMap<String, high_ast::TopLevelDecl>,
) -> Result<x64::Root, TypeResolveError> {
    let mut resolved_type_env: HashMap<String, PeachiliType> = Default::default();
    let mut constants: HashMap<String, x64::Constant> = Default::default();
    let mut functions: Vec<x64::Function> = Vec::with_capacity(common_root.decls.len());

    for common_tld in common_root.decls.iter() {
        match &common_tld.kind {
            high_ast::TopLevelDeclKind::PubType { type_name, to: _ } => {
                // コード生成用のASTに追加する必要はない
                let (r, _) = resolve_type(&raw_type_env, resolved_type_env, type_name)?;
                resolved_type_env = r;
            }
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
                let (r, x64_fn) = fn_to_lower(
                    &raw_type_env,
                    resolved_type_env,
                    func_name,
                    return_type,
                    parameters,
                    stmts,
                )?;
                resolved_type_env = r;
            }
            high_ast::TopLevelDeclKind::Import { module_name: _ } => {}
        }
    }

    Ok(x64::Root {
        functions: functions,
        constants,
    })
}

fn fn_to_lower(
    raw_type_env: &FxHashMap<String, high_ast::TopLevelDecl>,
    resolved_type_env: HashMap<String, PeachiliType>,
    func_name: &String,
    return_type: &String,
    parameters: &HashMap<String, String>,
    stmts: &[high_ast::StmtInfo],
) -> Result<(HashMap<String, PeachiliType>, x64::Function), TypeResolveError> {
    let (mut resolved_type_env, return_type) =
        resolve_type(raw_type_env, resolved_type_env, return_type)?;
    let mut local_variables: HashMap<String, x64::FrameObject> = Default::default();
    let mut fn_stack_size = 0;

    let lower_params = {
        let mut lower_params: HashMap<String, PeachiliType> = Default::default();
        for (param_name, param_type_name) in parameters.iter() {
            let (r, param_type) = resolve_type(raw_type_env, resolved_type_env, param_type_name)?;
            resolved_type_env = r;
            fn_stack_size += param_type.size();
            local_variables.insert(
                param_name.to_string(),
                x64::FrameObject {
                    stack_offset: fn_stack_size,
                    p_type: param_type.clone(),
                },
            );
            lower_params.insert(param_name.to_string(), param_type);
        }
        lower_params
    };

    let mut lower_stmts = Vec::new();
    for _stmt in stmts {
        lower_stmts.push(stmt_to_lower());
    }

    Ok((
        resolved_type_env,
        x64::Function {
            func_name: func_name.to_string(),
            return_type,
            params: lower_params,
            local_variables,
            stack_size: fn_stack_size,
            stmts: lower_stmts,
        },
    ))
}

fn stmt_to_lower() -> x64::Statement {
    x64::Statement::Nop
}

fn resolve_type(
    raw_type_env: &FxHashMap<String, high_ast::TopLevelDecl>,
    resolved_type_env: HashMap<String, PeachiliType>,
    type_name: &String,
) -> Result<(HashMap<String, PeachiliType>, PeachiliType), TypeResolveError> {
    match type_name.as_str() {
        "Int64" => Ok((resolved_type_env, PeachiliType::Int64)),
        "Uint64" => Ok((resolved_type_env, PeachiliType::Uint64)),
        "Noreturn" => Ok((resolved_type_env, PeachiliType::Noreturn)),
        "ConstStr" => Ok((resolved_type_env, PeachiliType::ConstStr)),
        _ => match raw_type_env.get(type_name) {
            Some(tld) => match &tld.kind {
                high_ast::TopLevelDeclKind::PubType {
                    type_name: alias_name,
                    to,
                } => {
                    let (mut resolved_type_env, alias_type) =
                        resolve_type(raw_type_env, resolved_type_env, to)?;
                    resolved_type_env.insert(alias_name.to_string(), alias_type.clone());

                    Ok((resolved_type_env, alias_type))
                }
                _ => Err(TypeResolveError::CannotResolve {
                    name: type_name.to_string(),
                }),
            },
            None => Err(TypeResolveError::CannotResolve {
                name: type_name.to_string(),
            }),
        },
    }
}

fn evaluate_constant_expr(
    const_name: &String,
    const_expr: &high_ast::ExprInfo,
) -> Result<x64::Constant, TypeResolveError> {
    match const_expr.kind {
        // 64bit整数の範囲を超えていたらとりあえずエラー
        high_ast::ExprKind::Integer { value } => {
            if value > (std::i64::MAX as i128) {
                Err(TypeResolveError::CannotEvaluateConst {
                    name: const_name.to_string(),
                })
            } else {
                Ok(x64::Constant::Integer(value as i64))
            }
        }
        high_ast::ExprKind::UnsignedInteger { value } => {
            if value > (std::u64::MAX as u128) {
                Err(TypeResolveError::CannotEvaluateConst {
                    name: const_name.to_string(),
                })
            } else {
                Ok(x64::Constant::UnsignedInteger(value as u64))
            }
        }
        _ => Err(TypeResolveError::CannotEvaluateConst {
            name: const_name.to_string(),
        }),
    }
}
