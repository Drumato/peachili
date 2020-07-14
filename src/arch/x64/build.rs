use crate::arch::x64;
use crate::common::module;
use crate::setup;

/// x64アーキテクチャ向けのビルドルーチン
pub fn main(main_module_id: module::ModuleId) -> Result<(), Box<dyn std::error::Error>> {
    x64::compiler::compile_main(
        setup::AST_FN_ARENA.clone(),
        setup::AST_STMT_ARENA.clone(),
        setup::AST_EXPR_ARENA.clone(),
        main_module_id
    );

    Ok(())
}
