use crate::common::pass::parser;
use crate::common::{ast, module};

/// フロントエンド資源をまとめる構造体
struct FrontendManager {
    full_ast: ast::ASTRoot,
}

/// 字句解析，パース，意味解析等を行う．
pub fn frontend<'a>(main_module: module::Module<'a>, debug: bool) -> ast::ASTRoot {
    let mut manager = FrontendManager {
        full_ast: Default::default(),
    };

    let source = manager.read_module_contents(main_module);

    // 初期値として空のStringを渡しておく
    manager.parse_file(source, String::new());

    // メインモジュールが参照する各モジュールも同様にパース
    manager.parse_requires(main_module, String::new());

    // ASTレベルのconstant-folding

    // TLD解析

    // 意味解析
    // 先に型環境を構築してから，型検査を行う

    if debug {
        // 型検査
    }

    // スタック割付
    // 通常はローカル変数をすべてスタックに．
    // 最適化を有効化にしたらレジスタ割付したい

    manager.full_ast
}

impl FrontendManager {
    /// モジュールの内容(Peachiliコード)を読み出す
    fn read_module_contents<'a>(&self, module_id: module::Module<'a>) -> String {
        unimplemented!()
    }

    /// 字句解析, 構文解析をして返す
    fn parse_file(&mut self, file_contents: String, module_name: String) {
        self.full_ast
            .absorb(parser::main(file_contents, module_name));
    }

    /// mod_idのモジュールが参照するすべてのモジュールをパースし，結合
    fn parse_requires<'a>(&mut self, m: module::Module<'a>, module_name: String) {
        unimplemented!()
    }

    /// 再帰呼出しされる，外部モジュールの組み立て関数
    /// 本体 -> 参照 -> 子の順にパースし，すべてを結合して返す
    fn parse_ext_module<'a>(&mut self, m: module::Module<'a>, mut module_name: String) {
        unimplemented!()
    }

    /// mのモジュール以下のすべてのモジュールをパースし，結合
    fn parse_children<'a>(&mut self, m: module::Module<'a>, module_name: String) {
        unimplemented!()
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
