use std::io::Write;

use crate::common::{arch, module, operate, option};
use crate::compiler::{pass, resource};

pub fn compile_main(
    build_option: &option::BuildOption,
    main_mod: module::Module,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: 所有権を考えつつ，どうやってrequires を処理するか．

    let assembly_file = process_main_module(build_option, &main_mod);

    if build_option.stop_assemble {
        // アセンブリファイルを生成してプロセスを終了
        // とりあえずAT&T syntaxで
        let mut asm_output = std::fs::File::create(&assembly_file.file_path).unwrap();
        asm_output.write_all(assembly_file.to_at_code().as_bytes())?;
        std::process::exit(0);
    }

    Ok(())
}

// TODO: とりあえずx64だけ
fn process_main_module(
    build_option: &option::BuildOption,
    main_mod: &module::Module,
) -> arch::x64::AssemblyFile {
    let contents = operate::read_program_from_file(&main_mod.file_path);
    let tokens = pass::tokenize(build_option, contents);

    dump_tokens_to_stderr(&tokens, build_option.debug);

    let functions = pass::parse(build_option, tokens);

    dump_functions_to_stderr(&functions, build_option.debug);

    pass::codegen::x64::codegen(build_option, functions)
}

fn dump_tokens_to_stderr(tokens: &[resource::Token], debug: bool) {
    if !debug {
        return;
    }

    eprintln!("++++++++ dump-token ++++++++");
    for t in tokens.iter() {
        eprintln!("\t{}", t);
    }
}

fn dump_functions_to_stderr(functions: &[resource::PFunction], debug: bool) {
    if !debug {
        return;
    }

    eprintln!("++++++++ dump-functions ++++++++");

    for f in functions.iter() {
        eprintln!("{}", f);
    }
}
