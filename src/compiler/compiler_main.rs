extern crate colored;
extern crate indicatif;

use std::collections::BTreeMap;
use std::time;

use colored::*;

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
    let contents = operate::read_program_from_file(&main_mod.file_path);

    // STEP1: tokenize
    let tokens = tokenize_phase(build_option, &main_mod.file_path, contents);

    // STEP2: parse
    let mut functions = parse_phase(build_option, &main_mod.file_path, tokens);

    for req_mod in main_mod.requires.borrow().iter() {
        let mut req_functions = proc_external_module(build_option, req_mod);
        functions.append(&mut req_functions);
    }

    // STEP3: 型検査(各関数内に入っていく)
    type_check_phase(build_option, &functions);

    // STEP4: スタックフレーム割付
    allocate_frame_phase(build_option, &mut functions);

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
    let contents = operate::read_program_from_file(&ext_mod.file_path);

    // STEP1: tokenize
    let tokens = tokenize_phase(build_option, &ext_mod.file_path, contents);

    // STEP2: parse
    parse_phase(build_option, &ext_mod.file_path, tokens)
}

fn tokenize_phase(
    build_option: &option::BuildOption,
    module_path: &str,
    contents: String,
) -> Vec<resource::Token> {
    let start = time::Instant::now();

    let (tokens, tokenize_errors) = pass::tokenize(build_option, contents);
    if !tokenize_errors.is_empty() {
        emit_all_errors_and_exit(&tokenize_errors, module_path, build_option);
    }

    let end = time::Instant::now();

    if build_option.verbose {
        eprintln!(
            "    {}: tokenize {} done in {:?}",
            "STEP1".bold().green(),
            module_path,
            end - start
        );
    }

    dump_tokens_to_stderr(&tokens, build_option.debug);

    tokens
}

fn parse_phase(
    build_option: &option::BuildOption,
    module_path: &str,
    tokens: Vec<resource::Token>,
) -> BTreeMap<String, resource::PFunction> {
    let start = time::Instant::now();
    let func_map = pass::parse(build_option, module_path, tokens);
    let end = time::Instant::now();

    // TODO: パースエラー

    dump_functions_to_stderr(&func_map, build_option.debug);

    if build_option.verbose {
        eprintln!(
            "    {}: parse {} done in {:?}",
            "STEP2".bold().green(),
            module_path,
            end - start
        );
    }
    func_map
}

fn type_check_phase(
    build_option: &option::BuildOption,
    func_map: &BTreeMap<String, resource::PFunction>,
) {
    let function_number = func_map.len() as u64;
    let type_check_pb = indicatif::ProgressBar::new(function_number);
    type_check_pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("#>-"),
    );

    let start = time::Instant::now();
    for (func_name, func) in func_map.iter() {
        type_check_pb.set_message(&format!("type check in {}", func_name));

        let errors = pass::type_check_fn(build_option, func_map, func);

        if !errors.is_empty() {
            let module_path = func.copy_module_path();
            emit_all_errors_and_exit(&errors, &module_path, build_option);
        }

        type_check_pb.inc(1);
    }
    let end = time::Instant::now();

    type_check_pb.finish_with_message(&format!("type check done!(in {:?})", end - start));
}

fn allocate_frame_phase(
    _build_option: &option::BuildOption,
    func_map: &mut BTreeMap<String, resource::PFunction>,
) {
    let function_number = func_map.len() as u64;
    let allocate_frame_pb = indicatif::ProgressBar::new(function_number);
    allocate_frame_pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("#>-"),
    );

    let start = time::Instant::now();
    for (func_name, func) in func_map.iter_mut() {
        allocate_frame_pb.set_message(&format!("allocate stack frame in {}", func_name));
        func.alloc_frame();

        allocate_frame_pb.inc(1);
    }
    let end = time::Instant::now();
    allocate_frame_pb
        .finish_with_message(&format!("allocate stack frame done!(in {:?})", end - start));
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
