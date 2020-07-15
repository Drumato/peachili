use crate::common::{compiler, module};

/// x64用コンパイラのメインルーチン
/// 機械独立なパスを呼び出した後x64依存のパスを処理する．
pub fn compile_main(
    module_arena: module::ModuleArena,
    main_module_id: module::ModuleId,
) {
    compiler::frontend(module_arena, main_module_id);
}
