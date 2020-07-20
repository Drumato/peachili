use crate::arch::x64;
use crate::common::module;

/// x64アーキテクチャ向けのビルドルーチン
pub fn main(module_arena: module::ModuleArena, main_module_id: module::ModuleId, verbose_ir: bool) -> Result<(), Box<dyn std::error::Error>> {
    x64::compiler::compile_main(
        module_arena,
        main_module_id,
        verbose_ir,
    );

    Ok(())
}
