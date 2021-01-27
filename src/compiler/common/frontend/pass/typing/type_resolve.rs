use crate::compiler::common::frontend::{ast as high_ast, frame_object, peachili_type};
use std::collections::{HashMap, VecDeque};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeResolveError {
    #[error("cannot resolve type of '{name:?}'")]
    CannotResolve { name: String },
}

pub fn type_resolve_main(
    ast_roots: &VecDeque<high_ast::ASTRoot>,
    raw_type_env: &HashMap<String, high_ast::TopLevelDecl>,
) -> Result<frame_object::GlobalEnv, Box<dyn std::error::Error>> {
    let mut global_env: frame_object::GlobalEnv = Default::default();
    global_env.initialize_predefined_type();

    for ast_root in ast_roots.iter() {
        for common_tld in ast_root.decls.iter() {
            match &common_tld.kind {
                high_ast::TopLevelDeclKind::PubType { type_name, to: _ } => {
                    resolve_type(
                        raw_type_env,
                        &mut global_env,
                        &ast_root.module_name,
                        &type_name,
                    )?;
                }
                high_ast::TopLevelDeclKind::Function {
                    func_name,
                    return_type,
                    parameters,
                    stmts: _,
                } => {
                    resolve_at_function(
                        raw_type_env,
                        &mut global_env,
                        func_name,
                        return_type,
                        &ast_root.module_name,
                        parameters,
                    )?;
                }
                high_ast::TopLevelDeclKind::Import { module_name: _ } => {}
                high_ast::TopLevelDeclKind::PubConst {
                    const_name: _,
                    const_type,
                    expr: _,
                } => {
                    let _const_type = resolve_type(
                        raw_type_env,
                        &mut global_env,
                        &ast_root.module_name,
                        &const_type,
                    )?;
                    // global_env.const_name_table.insert(const_name.to_string(), const_type);
                }
            }
        }
    }

    Ok(global_env)
}

fn resolve_at_function(
    raw_type_env: &HashMap<String, high_ast::TopLevelDecl>,
    global_env: &mut frame_object::GlobalEnv,
    func_name: &String,
    return_type_str: &String,
    ref_module_name: &String,
    parameters: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 関数の返り値の型解決
    let return_type = resolve_type(raw_type_env, global_env, ref_module_name, return_type_str)?;
    global_env
        .func_table
        .insert(func_name.to_string(), return_type.clone());

    for param in parameters.iter() {
        let _param_type = resolve_type(&raw_type_env, global_env, ref_module_name, param.1)?;
    }

    Ok(())
}

pub fn resolve_type(
    raw_type_env: &HashMap<String, high_ast::TopLevelDecl>,
    global_env: &mut frame_object::GlobalEnv,
    ref_module_name: &String,
    type_name: &String,
) -> Result<peachili_type::PeachiliType, TypeResolveError> {
    match find_typedef(raw_type_env, global_env, ref_module_name, type_name) {
        Ok(result) => Ok(result),
        Err(_e) => find_typedef(
            raw_type_env,
            global_env,
            ref_module_name,
            &format!("{}::{}", ref_module_name, type_name),
        ),
    }
}

pub fn find_typedef(
    raw_type_env: &HashMap<String, high_ast::TopLevelDecl>,
    global_env: &mut frame_object::GlobalEnv,
    ref_module_name: &String,
    type_name: &String,
) -> Result<peachili_type::PeachiliType, TypeResolveError> {
    if let Some(ty) = global_env.type_name_table.get(type_name) {
        return Ok(ty.clone());
    }
    match raw_type_env.get(type_name) {
        Some(tld) => match &tld.kind {
            high_ast::TopLevelDeclKind::PubType {
                type_name: alias_name,
                to,
            } => {
                let alias_type = resolve_type(raw_type_env, global_env, ref_module_name, to)?;
                global_env
                    .type_name_table
                    .insert(alias_name.to_string(), alias_type.clone());

                Ok(alias_type)
            }
            _ => Err(TypeResolveError::CannotResolve {
                name: type_name.to_string(),
            }),
        },
        _ => Err(TypeResolveError::CannotResolve {
            name: type_name.to_string(),
        }),
    }
}
