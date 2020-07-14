use crate::common::{compiler, module};
use crate::setup;

/// x64用コンパイラのメインルーチン
/// 機械独立なパスを呼び出した後x64依存のパスを処理する．
pub fn compile_main(
    fn_arena: setup::FnArena,
    stmt_arena: setup::StmtArena,
    expr_arena: setup::ExprArena,
    main_module_id: module::ModuleId
) {
    compiler::frontend(fn_arena, stmt_arena, expr_arena, main_module_id);
}
