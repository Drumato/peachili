use crate::common::{
    file_util,
    module,
};
use crate::common::compiler::{
    tokenizer
};
use crate::setup::MODULE_ARENA;

/// 字句解析，パース，意味解析等を行う．
pub fn frontend(main_module_id: module::ModuleId) {
    if let Ok(arena) = MODULE_ARENA.lock() {
        let main_module = arena.get(main_module_id).unwrap();
        let source = file_util::read_program_from_file(main_module.get_path());
        // Bundlerがファイルの存在はチェックしているはず
        assert!(source.is_some());

        let _tokens = tokenizer::main(source.unwrap());
    }
}