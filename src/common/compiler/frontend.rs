use crate::common::compiler::{analyzer, parser, tld_collector, tokenizer};
use crate::common::{ast, peachili_type, file_util, module};
use crate::setup;
use std::sync::{Mutex, Arc};
use id_arena::Arena;
use std::collections::BTreeMap;

/// 字句解析，パース，意味解析等を行う．
pub fn frontend(
    module_arena: module::ModuleArena,
    main_module_id: module::ModuleId,
) -> (ast::FnArena, ast::ASTRoot, BTreeMap<String, BTreeMap<String, peachili_type::Type>>) {
    let fn_arena = Arc::new(Mutex::new(Arena::new()));
    let source = read_module_contents(module_arena.clone(), main_module_id);

    // 初期値として空のStringを渡しておく
    let mut base_ast = parse_file(
        fn_arena.clone(),
        source,
        String::new(),
    );

    // メインモジュールが参照する各モジュールも同様にパース
    base_ast.absorb(parse_requires(
        module_arena,
        fn_arena.clone(),
        main_module_id,
        String::new(),
    ));

    // TLD解析
    let tld_env = tld_collector::main(fn_arena.clone(), &base_ast);

    // 意味解析
    // 先に型環境を構築してから，型検査を行う
    let type_env = analyzer::type_resolve_main(fn_arena.clone(), &tld_env, &base_ast, setup::BUILD_OPTION.target);
    analyzer::type_check_main(fn_arena.clone(), &tld_env, &type_env, &base_ast, setup::BUILD_OPTION.target);

    (fn_arena, base_ast, type_env)
}

/// 再帰呼出しされる，外部モジュールの組み立て関数
/// 本体 -> 参照 -> 子の順にパースし，すべてを結合して返す
fn parse_ext_module(
    module_arena: module::ModuleArena,
    fn_arena: ast::FnArena,
    ext_id: module::ModuleId,
    mut module_name: String,
) -> ast::ASTRoot {
    let is_dir_module = module_arena
        .lock()
        .unwrap()
        .get(ext_id)
        .unwrap()
        .child_count()
        != 0;
    // モジュール名を構築．
    let this_module_name = module_arena
        .lock()
        .unwrap()
        .get(ext_id)
        .unwrap()
        .copy_name();
    construct_full_path(&mut module_name, this_module_name);

    let mut base_ast = if is_dir_module {
        Default::default()
    } else {
        let source = read_module_contents(module_arena.clone(), ext_id);
        parse_file(
            fn_arena.clone(),
            source,
            module_name.clone(),
        )
    };

    // 参照･子ノードたちのパース，結合
    base_ast.absorb(parse_requires(
        module_arena.clone(),
        fn_arena.clone(),
        ext_id,
        module_name.clone(),
    ));
    base_ast.absorb(parse_children(
        module_arena,
        fn_arena,
        ext_id,
        module_name,
    ));

    base_ast
}

// mod_idのモジュールが参照するすべてのモジュールをパースし，結合
fn parse_requires(
    module_arena: module::ModuleArena,
    fn_arena: ast::FnArena,
    mod_id: module::ModuleId,
    module_name: String,
) -> ast::ASTRoot {
    let mut base_ast: ast::ASTRoot = Default::default();

    // 参照ノードをすべて取得
    let requires = module_arena
        .lock()
        .unwrap()
        .get(mod_id)
        .unwrap()
        .refs
        .clone();
    for req_id in requires.lock().unwrap().iter() {
        let req_ast = parse_ext_module(
            module_arena.clone(),
            fn_arena.clone(),
            *req_id,
            module_name.clone(),
        );
        base_ast.absorb(req_ast);
    }

    base_ast
}

// mod_idのモジュール以下のすべてのモジュールをパースし，結合
fn parse_children(
    module_arena: module::ModuleArena,
    fn_arena: ast::FnArena,
    mod_id: module::ModuleId,
    module_name: String,
) -> ast::ASTRoot {
    let mut base_ast: ast::ASTRoot = Default::default();

    // 参照ノードをすべて取得
    let children = module_arena
        .lock()
        .unwrap()
        .get(mod_id)
        .unwrap()
        .children
        .clone();
    for child_id in children.lock().unwrap().iter() {
        let child_ast = parse_ext_module(
            module_arena.clone(),
            fn_arena.clone(),
            *child_id,
            module_name.clone(),
        );
        base_ast.absorb(child_ast);
    }

    base_ast
}

// 字句解析, 構文解析をして返す
fn parse_file(
    fn_arena: ast::FnArena,
    file_contents: String,
    module_name: String,
) -> ast::ASTRoot {
    let tokens = tokenizer::main(file_contents);

    parser::main(fn_arena, tokens, module_name)
}

// モジュールの内容(Peachiliコード)を読み出す
fn read_module_contents(
    module_arena: module::ModuleArena,
    module_id: module::ModuleId,
) -> String {
    if let Ok(arena) = module_arena.lock() {
        let main_module = arena.get(module_id).unwrap();
        let source = file_util::read_program_from_file(main_module.get_path());

        // Bundlerがファイルの存在はチェックしているはず
        assert!(source.is_some());

        return source.unwrap();
    }

    unreachable!()
}

// トップのモジュールなら `std` のように
// それ以降なら `std::os` のようにつなげる
fn construct_full_path(full_path: &mut String, module_name: String) {
    *full_path = if full_path.is_empty() {
        module_name
    } else {
        format!("{}::{}", full_path, module_name)
    };
}

#[cfg(test)]
mod frontend_tests {
    use super::*;
    use id_arena::Arena;
    use std::sync::{Arc, Mutex};

    #[test]
    fn construct_full_path_test() {
        let mut s1 = String::new();
        construct_full_path(&mut s1, "std".to_string());
        assert_eq!("std", s1);

        let mut s2 = String::from("std");
        construct_full_path(&mut s2, "os".to_string());
        assert_eq!("std::os", s2);
    }

    #[test]
    fn parse_file_test() {
        let f = new_allocators();
        let a = parse_file(
            f,
            "func main() Noreturn {}".to_string(),
            "sample".to_string(),
        );
        assert_eq!(1, a.funcs.len());
        assert_eq!(0, a.typedefs.len());
        assert_eq!(0, a.alias.len());
    }

    fn new_allocators() -> ast::FnArena {
        Arc::new(Mutex::new(Arena::new()))
    }
}
