use crate::compiler::common::frontend::{allocator, pass, types};
use crate::compiler::common::hir;
use crate::{module, option};
use fxhash::FxHashMap;
use types::ast;

use std::collections::VecDeque;

/// 字句解析，パース，意味解析等を行う．
pub fn main(
    main_module: module::Module,
    build_option: option::BuildOption,
) -> Result<(), Box<dyn std::error::Error>> {
    let module_queue = pseudo_topological_sort_modules(main_module);
    let mut hir_program_queue: VecDeque<hir::Program> = VecDeque::new();

    let mut raw_type_env: FxHashMap<String, ast::TopLevelDecl> = FxHashMap::default();
    let alloc: allocator::Allocator = Default::default();
    let parser = pass::parser::Parser::new(&alloc);

    for m in module_queue.iter() {
        // キューにはPrimitiveモジュールしか存在しない
        if let module::ModuleKind::Primitive { refs: _, contents } = &m.kind {
            // 初期値として空のStringを渡しておく
            let ast_root = pass::parser::main(&parser, &m.name, contents.as_str()).unwrap();
            if build_option.dump_ir {
                ast::dump_ast_root(&ast_root);
            }

            for decl in ast_root.decls.iter() {
                if let Some((decl_name, copied_decl)) = copy_tld_by(decl.clone()) {
                    raw_type_env.insert(decl_name, copied_decl);
                }
            }
        }
    }
    // ASTレベルのconstant-folding

    // TLD解析

    // 意味解析
    // 先に型環境を構築してから，型検査を行う

    // 型検査

    // スタック割付
    // 通常はローカル変数をすべてスタックに．
    // 最適化を有効化にしたらレジスタ割付したい

    Ok(())
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
            for ref_module in refs.lock().unwrap().iter() {
                let mut ref_queue = collect_module_rec(ref_module);
                queue.append(&mut ref_queue);
            }

            queue.push_back(base_module);
        }

        // Directory モジュールの場合自分自身はキューに追加しないので注意
        module::ModuleKind::Directory { children } => {
            for child in children.lock().unwrap().iter() {
                let mut child_queue = collect_module_rec(child);
                queue.append(&mut child_queue);
            }
        }
    }

    queue
}

fn copy_tld_by<'a>(decl: ast::TopLevelDecl<'a>) -> Option<(String, ast::TopLevelDecl<'a>)> {
    match &decl.kind {
        ast::TopLevelDeclKind::Import { module_name: _ } => None,
        ast::TopLevelDeclKind::Function {
            func_name,
            return_type: _,
            stmts: _,
        } => Some((func_name.to_string(), decl.clone())),
    }
}

#[cfg(test)]
mod frontend_tests {}
