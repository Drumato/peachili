use crate::common::compiler::{parser, tokenizer};
use crate::common::{file_util, module};
use crate::setup;

/// 字句解析，パース，意味解析等を行う．
pub fn frontend(main_module_id: module::ModuleId) {
    if let Ok(arena) = setup::MODULE_ARENA.lock() {
        let main_module = arena.get(main_module_id).unwrap();
        let source = file_util::read_program_from_file(main_module.get_path());
        // Bundlerがファイルの存在はチェックしているはず
        assert!(source.is_some());

        let tokens = tokenizer::main(source.unwrap());

        // mainモジュールでは空のVecを渡しておく
        let _ast = parser::main(
            setup::AST_FN_ARENA.clone(),
            setup::AST_STMT_ARENA.clone(),
            setup::AST_EXPR_ARENA.clone(),
            tokens,
            vec![]
        );
    }
}
