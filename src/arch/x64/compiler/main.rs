use crate::common::{compiler, module};
use crate::debug;

/// x64用コンパイラのメインルーチン
/// 機械独立なパスを呼び出した後x64依存のパスを処理する．
pub fn compile_main(
    module_arena: module::ModuleArena,
    main_module_id: module::ModuleId,
    verbose_ir: bool,
) {
    let (fn_arena, ast_root, type_env) = compiler::frontend(module_arena, main_module_id);
    let ir_module = compiler::translate_ir(fn_arena, ast_root, &type_env);

    if verbose_ir {
        debug::dump_hir(&ir_module);
    }
}
