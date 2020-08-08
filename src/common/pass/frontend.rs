use crate::common::pass::{analyzer, parser, tld_collector, tokenizer};
use crate::common::{ast, file_util, frame_object, module, peachili_type};
use crate::setup;
use id_arena::Arena;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// フロントエンド資源をまとめる構造体
struct FrontendManager {
    module_arena: module::ModuleArena,
    fn_arena: ast::FnArena,
    full_ast: ast::ASTRoot,
}

/// 字句解析，パース，意味解析等を行う．
pub fn frontend(
    module_arena: module::ModuleArena,
    main_module_id: module::ModuleId,
    debug: bool,
) -> (
    ast::FnArena,
    ast::ASTRoot,
    BTreeMap<String, BTreeMap<String, peachili_type::Type>>,
    frame_object::StackFrame,
) {
    let mut manager = FrontendManager {
        module_arena,
        fn_arena: Arc::new(Mutex::new(Arena::new())),
        full_ast: Default::default(),
    };

    let source = manager.read_module_contents(main_module_id);

    // 初期値として空のStringを渡しておく
    manager.parse_file(source, String::new());

    // メインモジュールが参照する各モジュールも同様にパース
    manager.parse_requires(main_module_id, String::new());

    // TLD解析
    let tld_env = tld_collector::main(manager.fn_arena.clone(), &manager.full_ast);

    // 意味解析
    // 先に型環境を構築してから，型検査を行う
    let type_env = analyzer::type_resolve_main(
        manager.fn_arena.clone(),
        &tld_env,
        &manager.full_ast,
        setup::BUILD_OPTION.target,
    );

    if debug {
        analyzer::type_check_main(
            manager.fn_arena.clone(),
            &tld_env,
            &type_env,
            &manager.full_ast,
            setup::BUILD_OPTION.target,
        );
    }

    // スタック割付
    // 通常はローカル変数をすべてスタックに．
    // 最適化を有効化にしたらレジスタ割付したい
    let func_frame = analyzer::allocate_stack_frame(&tld_env, &type_env);

    (manager.fn_arena, manager.full_ast, type_env, func_frame)
}

impl FrontendManager {
    /// モジュールの内容(Peachiliコード)を読み出す
    fn read_module_contents(&self, module_id: module::ModuleId) -> String {
        if let Ok(arena) = self.module_arena.lock() {
            let m = arena.get(module_id).unwrap();
            let source = file_util::read_program_from_file(m.get_path());

            // Bundlerがファイルの存在はチェックしているはず
            assert!(source.is_some());

            return source.unwrap();
        }

        unreachable!()
    }

    /// 字句解析, 構文解析をして返す
    fn parse_file(&mut self, file_contents: String, module_name: String) {
        let tokens = tokenizer::main(file_contents);

        self.full_ast
            .absorb(parser::main(self.fn_arena.clone(), tokens, module_name));
    }

    /// mod_idのモジュールが参照するすべてのモジュールをパースし，結合
    fn parse_requires(&mut self, mod_id: module::ModuleId, module_name: String) {
        // 参照ノードをすべて取得
        let requires = self
            .module_arena
            .lock()
            .unwrap()
            .get(mod_id)
            .unwrap()
            .refs
            .clone();
        for req_id in requires.lock().unwrap().iter() {
            self.parse_ext_module(*req_id, module_name.clone());
        }
    }

    /// 再帰呼出しされる，外部モジュールの組み立て関数
    /// 本体 -> 参照 -> 子の順にパースし，すべてを結合して返す
    fn parse_ext_module(&mut self, ext_id: module::ModuleId, mut module_name: String) {
        let is_dir_module = self
            .module_arena
            .lock()
            .unwrap()
            .get(ext_id)
            .unwrap()
            .child_count()
            != 0;
        // モジュール名を構築．
        let this_module_name = self
            .module_arena
            .lock()
            .unwrap()
            .get(ext_id)
            .unwrap()
            .copy_name();
        construct_full_path(&mut module_name, this_module_name);

        if !is_dir_module {
            let source = self.read_module_contents(ext_id);
            self.parse_file(source, module_name.clone());
        }

        // 参照･子ノードたちのパース，結合
        self.parse_requires(ext_id, module_name.clone());
        self.parse_children(ext_id, module_name);
    }

    /// mod_idのモジュール以下のすべてのモジュールをパースし，結合
    fn parse_children(&mut self, mod_id: module::ModuleId, module_name: String) {
        // 参照ノードをすべて取得
        let children = self
            .module_arena
            .lock()
            .unwrap()
            .get(mod_id)
            .unwrap()
            .children
            .clone();
        for child_id in children.lock().unwrap().iter() {
            self.parse_ext_module(*child_id, module_name.clone());
        }
    }
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

    #[test]
    fn construct_full_path_test() {
        let mut s1 = String::new();
        construct_full_path(&mut s1, "std".to_string());
        assert_eq!("std", s1);

        let mut s2 = String::from("std");
        construct_full_path(&mut s2, "os".to_string());
        assert_eq!("std::os", s2);
    }
}
