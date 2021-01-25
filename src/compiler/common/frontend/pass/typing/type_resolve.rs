use crate::compiler::common::frontend::{ast as high_ast, peachili_type};
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
) -> Result<HashMap<String, peachili_type::PeachiliType>, Box<dyn std::error::Error>> {
    let mut type_env: HashMap<String, peachili_type::PeachiliType> = Default::default();

    // プリミティブな型はそのまま変換するだけ
    type_env.insert(
        "Int64".to_string(),
        peachili_type::PeachiliType::new(peachili_type::PTKind::Int64, 8),
    );
    type_env.insert(
        "Uint64".to_string(),
        peachili_type::PeachiliType::new(peachili_type::PTKind::Uint64, 8),
    );
    type_env.insert(
        "Noreturn".to_string(),
        peachili_type::PeachiliType::new(peachili_type::PTKind::Noreturn, 0),
    );
    type_env.insert(
        "ConstStr".to_string(),
        peachili_type::PeachiliType::new(peachili_type::PTKind::Noreturn, 0),
    );
    type_env.insert(
        "Boolean".to_string(),
        peachili_type::PeachiliType::new(peachili_type::PTKind::Boolean, 0),
    );

    for ast_root in ast_roots.iter() {
        for common_tld in ast_root.decls.iter() {
            match &common_tld.kind {
                high_ast::TopLevelDeclKind::PubType { type_name, to: _ } => {
                    let (r, _) =
                        resolve_type(raw_type_env, type_env, &ast_root.module_name, &type_name)?;
                    type_env = r;
                }
                high_ast::TopLevelDeclKind::Function {
                    func_name,
                    return_type,
                    parameters,
                    stmts: _,
                } => {
                    type_env = resolve_at_function(
                        raw_type_env,
                        type_env,
                        func_name,
                        return_type,
                        &ast_root.module_name,
                        parameters,
                    )?;
                }
                high_ast::TopLevelDeclKind::Import { module_name: _ } => {}
                high_ast::TopLevelDeclKind::PubConst {
                    const_name: _,
                    const_type: _,
                    expr: _,
                } => {}
            }
        }
    }

    Ok(type_env)
}

fn resolve_at_function(
    raw_type_env: &HashMap<String, high_ast::TopLevelDecl>,
    type_env: HashMap<String, peachili_type::PeachiliType>,
    func_name: &String,
    return_type_str: &String,
    ref_module_name: &String,
    parameters: &HashMap<String, String>,
) -> Result<HashMap<String, peachili_type::PeachiliType>, Box<dyn std::error::Error>> {
    // 関数の返り値の型解決
    let (mut type_env, return_type) = resolve_type(
        raw_type_env,
        type_env.clone(),
        ref_module_name,
        return_type_str,
    )?;
    type_env.insert(func_name.to_string(), return_type.clone());

    for param in parameters.iter() {
        let (t, _param_type) = resolve_type(&raw_type_env, type_env, ref_module_name, param.1)?;
        type_env = t;
    }

    Ok(type_env)
}

pub fn resolve_type(
    raw_type_env: &HashMap<String, high_ast::TopLevelDecl>,
    resolved_type_env: HashMap<String, peachili_type::PeachiliType>,
    ref_module_name: &String,
    type_name: &String,
) -> Result<
    (
        HashMap<String, peachili_type::PeachiliType>,
        peachili_type::PeachiliType,
    ),
    TypeResolveError,
> {
    match type_name.as_str() {
        // プリミティブな型はそのまま変換するだけ
        "Int64" => Ok((
            resolved_type_env,
            peachili_type::PeachiliType::new(peachili_type::PTKind::Int64, 8),
        )),
        "Uint64" => Ok((
            resolved_type_env,
            peachili_type::PeachiliType::new(peachili_type::PTKind::Uint64, 8),
        )),
        "Noreturn" => Ok((
            resolved_type_env,
            peachili_type::PeachiliType::new(peachili_type::PTKind::Noreturn, 0),
        )),
        "ConstStr" => Ok((
            resolved_type_env,
            peachili_type::PeachiliType::new(peachili_type::PTKind::ConstStr, 8),
        )),
        "Boolean" => Ok((
            resolved_type_env,
            peachili_type::PeachiliType::new(peachili_type::PTKind::Boolean, 8),
        )),

        // 識別子の場合
        // "(定義箇所と異なるモジュールから)実際に使用される名前" を調べた後，
        // "同一モジュール内で定義された型名" の場合も調べる．
        // そのためにref_module_nameを用いる．
        _ => match find_typedef(
            raw_type_env,
            resolved_type_env.clone(),
            ref_module_name,
            type_name,
        ) {
            Ok(result) => Ok(result),
            Err(_e) => find_typedef(
                raw_type_env,
                resolved_type_env,
                ref_module_name,
                &format!("{}::{}", ref_module_name, type_name),
            ),
        },
    }
}

pub fn find_typedef(
    raw_type_env: &HashMap<String, high_ast::TopLevelDecl>,
    resolved_type_env: HashMap<String, peachili_type::PeachiliType>,
    ref_module_name: &String,
    type_name: &String,
) -> Result<
    (
        HashMap<String, peachili_type::PeachiliType>,
        peachili_type::PeachiliType,
    ),
    TypeResolveError,
> {
    match raw_type_env.get(type_name) {
        Some(tld) => match &tld.kind {
            high_ast::TopLevelDeclKind::PubType {
                type_name: alias_name,
                to,
            } => {
                let (mut resolved_type_env, alias_type) =
                    resolve_type(raw_type_env, resolved_type_env, ref_module_name, to)?;
                resolved_type_env.insert(alias_name.to_string(), alias_type.clone());

                Ok((resolved_type_env, alias_type))
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
