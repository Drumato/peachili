use std::collections::HashMap;

use fxhash::FxHashMap;

use crate::compiler::{arch::x64, common::frontend};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeResolveError {
    #[error("cannot resolve type of '{name:?}'")]
    CannotResolve { name: String },
    #[error("cannot evaluate '{name:?}' as a const now")]
    CannotEvaluateConst { name: String },
}
pub fn resolve_type(
    raw_type_env: &FxHashMap<String, frontend::ast::TopLevelDecl>,
    resolved_type_env: HashMap<String, x64::PeachiliType>,
    ref_module_name: &String,
    type_name: &String,
) -> Result<(HashMap<String, x64::PeachiliType>, x64::PeachiliType), TypeResolveError> {
    match type_name.as_str() {
        // プリミティブな型はそのまま変換するだけ
        "Int64" => Ok((resolved_type_env, x64::PeachiliType::Int64)),
        "Uint64" => Ok((resolved_type_env, x64::PeachiliType::Uint64)),
        "Noreturn" => Ok((resolved_type_env, x64::PeachiliType::Noreturn)),
        "ConstStr" => Ok((resolved_type_env, x64::PeachiliType::ConstStr)),
        "Boolean" => Ok((resolved_type_env, x64::PeachiliType::Boolean)),

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
    raw_type_env: &FxHashMap<String, frontend::ast::TopLevelDecl>,
    resolved_type_env: HashMap<String, x64::PeachiliType>,
    ref_module_name: &String,
    type_name: &String,
) -> Result<(HashMap<String, x64::PeachiliType>, x64::PeachiliType), TypeResolveError> {
    match raw_type_env.get(type_name) {
        Some(tld) => match &tld.kind {
            frontend::ast::TopLevelDeclKind::PubType {
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
