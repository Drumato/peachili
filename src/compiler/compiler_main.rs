use std::io::Write;

use crate::common::{arch, module, operate, option};
use crate::compiler::{pass, resource};

pub fn compile_main(
    build_option: &option::BuildOption,
    main_mod: module::Module,
) -> Result<(), Box<dyn std::error::Error>> {
    if build_option.verbose {
        eprintln!("start compiling...");
    }

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
    if build_option.verbose {
        eprintln!("process {} module...", main_mod.file_path);
    }

    let contents = operate::read_program_from_file(&main_mod.file_path);
    let tokens = pass::tokenize(build_option, contents);

    dump_tokens_to_stderr(&tokens, build_option.debug);

    let mut functions = pass::parse(build_option, tokens);

    for req_mod in main_mod.requires.borrow().iter() {
        let mut req_functions = proc_external_module(build_option, req_mod);
        functions.append(&mut req_functions);
    }

    dump_functions_to_stderr(&functions, build_option.debug);

    for func in functions.iter_mut() {
        func.alloc_frame();
    }

    pass::codegen::x64::codegen(build_option, functions)
}

fn proc_external_module(
    build_option: &option::BuildOption,
    ext_mod: &module::Module,
) -> Vec<resource::PFunction> {
    if build_option.verbose {
        eprintln!("process {} module...", ext_mod.file_path);
    }

    if ext_mod.subs.borrow().len() == 0 {
        let req_contents = operate::read_program_from_file(&ext_mod.file_path);
        let req_tokens = pass::tokenize(build_option, req_contents);
        dump_tokens_to_stderr(&req_tokens, build_option.debug);
        let req_functions = pass::parse(build_option, req_tokens);
        dump_functions_to_stderr(&req_functions, build_option.debug);
        return req_functions;
    }
    let mut all_subs_functions: Vec<resource::PFunction> = Vec::new();

    for sub in ext_mod.subs.borrow().iter() {
        let mut sub_functions = proc_external_module(build_option, sub);
        all_subs_functions.append(&mut sub_functions);
    }
    all_subs_functions
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
