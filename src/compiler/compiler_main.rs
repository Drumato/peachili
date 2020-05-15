use std::collections::BTreeMap;

use crate::common::{arch, error, module, operate, option};
use crate::compiler::{pass, resource};

pub fn compile_main(
    build_option: &option::BuildOption,
    main_mod: module::Module,
) -> Result<arch::x64::AssemblyFile, Box<dyn std::error::Error>> {
    if build_option.verbose {
        eprintln!("start compiling...");
    }

    let assembly_file = process_main_module(build_option, &main_mod);

    Ok(assembly_file)
}

// TODO: とりあえずx64だけ
fn process_main_module(
    build_option: &option::BuildOption,
    main_mod: &module::Module,
) -> arch::x64::AssemblyFile {
    if build_option.verbose {
        eprintln!("process main module ->  {}:", main_mod.file_path);
    }

    let contents = operate::read_program_from_file(&main_mod.file_path);

    // STEP1: tokenize
    if build_option.verbose {
        eprintln!("\ttokenize start.");
    }
    let (tokens, tokenize_errors) = pass::tokenize(build_option, contents);
    if !tokenize_errors.is_empty() {
        emit_all_errors_and_exit(&tokenize_errors, &main_mod.file_path, build_option);
    }

    dump_tokens_to_stderr(&tokens, build_option.debug);

    // STEP2: parse
    if build_option.verbose {
        eprintln!("\tparse start.");
    }
    let mut functions = pass::parse(build_option, tokens);

    for req_mod in main_mod.requires.borrow().iter() {
        let mut req_functions = proc_external_module(build_option, req_mod);
        functions.append(&mut req_functions);
    }

    dump_functions_to_stderr(&functions, build_option.debug);

    // STEP3: 型検査(各関数内に入っていく)
    if build_option.verbose {
        eprintln!("\ttype_check start.");
    }
    for (_func_name, func) in functions.iter() {
        let errors = pass::type_check_fn(build_option, &functions, func);

        if !errors.is_empty() {
            emit_all_errors_and_exit(&errors, &main_mod.file_path, build_option);
        }
    }

    // STEP4: スタックフレーム割付
    if build_option.verbose {
        eprintln!("\tallocating stack frame.");
    }
    for (_func_name, func) in functions.iter_mut() {
        func.alloc_frame();
    }

    // STEP5: コード生成
    if build_option.verbose {
        eprintln!("\tgenerating x64 assembly start.");
    }
    pass::codegen::x64::codegen(build_option, functions)
}

fn proc_external_module(
    build_option: &option::BuildOption,
    ext_mod: &module::Module,
) -> BTreeMap<String, resource::PFunction> {
    if build_option.verbose {
        eprintln!("process {} module...", ext_mod.file_path);
    }

    if ext_mod.is_parent() {
        // サブモジュールをすべて処理して，返す
        let mut all_subs_functions: BTreeMap<String, resource::PFunction> = BTreeMap::new();

        for sub in ext_mod.subs.borrow().iter() {
            let mut sub_functions = proc_external_module(build_option, sub);
            all_subs_functions.append(&mut sub_functions);
        }
        return all_subs_functions;
    }

    // 末端モジュールの場合
    let req_contents = operate::read_program_from_file(&ext_mod.file_path);

    // STEP1: tokenize
    let (req_tokens, tokenize_errors) = pass::tokenize(build_option, req_contents);
    if !tokenize_errors.is_empty() {
        emit_all_errors_and_exit(&tokenize_errors, &ext_mod.file_path, build_option);
    }
    dump_tokens_to_stderr(&req_tokens, build_option.debug);

    // STEP2: parse
    let req_functions = pass::parse(build_option, req_tokens);
    dump_functions_to_stderr(&req_functions, build_option.debug);

    req_functions
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

fn dump_functions_to_stderr(functions: &BTreeMap<String, resource::PFunction>, debug: bool) {
    if !debug {
        return;
    }

    eprintln!("++++++++ dump-functions ++++++++");

    for (_name, f) in functions.iter() {
        eprintln!("{}", f);
    }
}

fn emit_all_errors_and_exit(
    errors: &[error::CompileError],
    module_path: &str,
    build_opt: &option::BuildOption,
) -> ! {
    for err in errors.iter() {
        err.emit_stderr(module_path, build_opt);
    }

    std::process::exit(1);
}
