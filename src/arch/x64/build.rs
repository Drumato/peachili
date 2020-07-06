use crate::arch::x64;
use crate::common::module;

/// x64アーキテクチャ向けのビルドルーチン
pub fn main(main_module_id: module::ModuleId) -> Result<(), Box<dyn std::error::Error>> {
    x64::compiler::main(main_module_id);

    Ok(())
}
