// use std::io::Write;

use crate::{
    common::{module, option},
    compiler::llvm_compiler,
};

pub fn main(
    build_option: &option::BuildOption,
    main_mod_id: module::ModuleId,
    module_allocator: module::ModuleAllocator,
) -> Result<(), Box<dyn std::error::Error>> {
    llvm_compiler::compile_main(&build_option, main_mod_id, &module_allocator);

    // IRファイルを生成
    // let mut ir_output = std::fs::File::create(&ir_file.file_path).unwrap();
    // ir_output.write_all(ir_file.to_code().as_bytes())?;

    Ok(())
}
