use std::io::Write;

use crate::{
    assembler::x64 as x64_assembler,
    common::{module, option},
    compiler::x64_compiler,
};

pub fn main(
    build_option: &option::BuildOption,
    main_mod: module::Module,
) -> Result<(), Box<dyn std::error::Error>> {
    let assembly_file = x64_compiler::compile_main(&build_option, main_mod);

    if build_option.stop_assemble {
        // アセンブリファイルを生成してプロセスを終了
        // とりあえずAT&T syntaxで
        let mut asm_output = std::fs::File::create(&assembly_file.file_path).unwrap();
        asm_output.write_all(assembly_file.to_at_code().as_bytes())?;
        std::process::exit(0);
    }

    // *****************
    // *   Assembler   *
    // *****************
    let elf_builder = x64_assembler::x64_assemble(&build_option, assembly_file);

    if build_option.stop_link {
        // オブジェクトファイルを生成して終了
        elf_builder.generate_elf_file("obj.o");
    }

    Ok(())
}
