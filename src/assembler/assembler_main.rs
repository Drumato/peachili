use crate::assembler::x64::assembler;
use crate::common::*;

pub fn x64_assemble(build_option: &option::BuildOption, asm_file: arch::x64::AssemblyFile) {
    if build_option.verbose {
        eprintln!("start assembling...");
    }

    let mut generator: assembler::Assembler = Default::default();
    generator.codegen(&asm_file);

    if build_option.stop_link {
        // オブジェクトファイルを生成して終了
        panic!("generating an object file feature is not implemented");
    }
}
