use crate::common::{compiler, module};
use crate::setup;

/// x64用コンパイラのメインルーチン
/// 機械独立なパスを呼び出した後x64依存のパスを処理する．
pub fn compile_main(
    fn_arena: setup::FnArena,
    main_module_id: module::ModuleId
) {
    compiler::frontend(fn_arena, main_module_id);
}
