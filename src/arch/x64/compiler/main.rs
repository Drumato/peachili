use crate::common::{compiler, module};
use crate::debug;
use colored::*;

/// x64用コンパイラのメインルーチン
/// 機械独立なパスを呼び出した後x64依存のパスを処理する．
pub fn compile_main(
    module_arena: module::ModuleArena,
    main_module_id: module::ModuleId,
    verbose_ir: bool,
    debug: bool,
) {
    let (fn_arena, ast_root, type_env, _stack_frame) = compiler::frontend(module_arena, main_module_id, debug);
    let ir_module = compiler::translate_ir(fn_arena, ast_root, &type_env);

    if verbose_ir {
        eprintln!("{}", "dump HIR to 'hir_dump'...".bold().blue());
        debug::dump_hir(&ir_module);
        eprintln!("{}", "done!".bold().blue());
    }

    // BasicBlockのない，ローカルなグラフを作成する
    let local_cfg = compiler::build_local_cfg(&ir_module);

    if verbose_ir {
        eprintln!("{}", "dump CFG to 'local_cfg.dot' ...".bold().blue());
        debug::dump_local_cfg("local_cfg.dot", &ir_module, &local_cfg);
        eprintln!("{}", "done!".bold().blue());
    }

}
