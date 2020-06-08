use std::io::Write;

use crate::{
    assembler::x64 as x64_assembler,
    linker::x64 as x64_linker,
    common::{module, option},
    compiler::{x64_compiler},
};

pub fn main(
    build_option: &option::BuildOption,
    main_mod_id: module::ModuleId,
    module_allocator: module::ModuleAllocator,
) -> Result<(), Box<dyn std::error::Error>> {
    let assembly_file = x64_compiler::compile_main(&build_option, main_mod_id, &module_allocator);

    if build_option.stop_assemble {
        // アセンブリファイルを生成
        // とりあえずAT&T syntaxで
        let mut asm_output = std::fs::File::create(&assembly_file.file_path).unwrap();
        asm_output.write_all(assembly_file.to_at_code().as_bytes())?;

        return Ok(());
    }

    // *****************
    // *   Assembler   *
    // *****************
    let object_file_builder = x64_assembler::x64_assemble(&build_option, assembly_file);

    if build_option.stop_link {
        // オブジェクトファイルを生成して終了
        object_file_builder.generate_elf_file("obj.o", 0o644);
    }

    let executable_elf_builder = x64_linker::x64_static_link(&build_option, object_file_builder);
    executable_elf_builder.generate_elf_file("a.out", 0o755);

    Ok(())
}
