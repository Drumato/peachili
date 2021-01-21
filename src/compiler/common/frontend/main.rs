use crate::compiler::common::frontend::{pass, types};
use crate::{module, option};
use types::ast;

use std::collections::{HashMap, VecDeque};

/// 字句解析，パース，意味解析等を行う．
pub fn main<'a>(
    main_module: module::Module<'a>,
    build_option: option::BuildOption,
) -> Result<
    (VecDeque<ast::ASTRoot>, HashMap<String, ast::TopLevelDecl>),
    Box<dyn std::error::Error + 'a>,
> {
    let module_queue = pseudo_topological_sort_modules(main_module);
    let mut ast_root_deque: VecDeque<ast::ASTRoot> = VecDeque::new();

    let mut raw_type_env: HashMap<String, ast::TopLevelDecl> = Default::default();

    for m in module_queue.iter() {
        // キューにはPrimitiveモジュールしか存在しない
        if let module::ModuleKind::Primitive { refs: _, contents } = &m.kind {
            // 初期値として空のStringを渡しておく
            let ast_root = pass::parser::main(&m.name, contents.as_str())?;
            if build_option.dump_ir {
                ast::dump_ast_root(&ast_root);
            }

            for decl in ast_root.decls.iter() {
                if let Some((decl_name, copied_decl)) = copy_tld_by(decl.clone()) {
                    raw_type_env.insert(decl_name, copied_decl);
                }
            }
            ast_root_deque.push_back(ast_root);
        }
    }
    Ok((ast_root_deque, raw_type_env))
}

/// モジュールの依存関係をソートしてキューを作成．
/// 現在は特に考えず，シンプルな関数で実装
fn pseudo_topological_sort_modules<'a>(
    main_module: module::Module<'a>,
) -> VecDeque<module::Module<'a>> {
    collect_module_rec(main_module)
}

fn collect_module_rec<'a>(base_module: module::Module<'a>) -> VecDeque<module::Module<'a>> {
    let mut queue = VecDeque::new();

    match &base_module.kind {
        module::ModuleKind::Primitive { contents: _, refs } => {
            for ref_module in refs.as_ref().borrow().iter() {
                let mut ref_queue = collect_module_rec(ref_module);
                queue.append(&mut ref_queue);
            }

            queue.push_back(base_module);
        }

        // Directory モジュールの場合自分自身はキューに追加しないので注意
        module::ModuleKind::Directory { children } => {
            for child in children.as_ref().borrow().iter() {
                let mut child_queue = collect_module_rec(child);
                queue.append(&mut child_queue);
            }
        }
    }
    queue
}

fn copy_tld_by<'a>(decl: ast::TopLevelDecl) -> Option<(String, ast::TopLevelDecl)> {
    match &decl.kind {
        ast::TopLevelDeclKind::Import { module_name: _ } => None,
        ast::TopLevelDeclKind::Function {
            func_name,
            return_type: _,
            parameters: _,
            stmts: _,
        } => Some((func_name.to_string(), decl.clone())),
        ast::TopLevelDeclKind::PubType { type_name, to: _ } => {
            Some((type_name.to_string(), decl.clone()))
        }
        ast::TopLevelDeclKind::PubConst {
            const_name,
            const_type: _,
            expr: _,
        } => Some((const_name.to_string(), decl.clone())),
    }
}

#[cfg(test)]
mod frontend_tests {}
