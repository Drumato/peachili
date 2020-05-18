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
    let mut root = parse_phase(build_option, &main_mod.file_path, tokens);

    for req_mod in main_mod.requires.borrow().iter() {
        let req_root = proc_external_module(build_option, req_mod);
        root.append(req_root);
    }

    // STEP3: resolve TLD
    // TODO: see Issue#12
    let tld_map = resolve_tld_phase(build_option, &root);

    // STEP4: 型検査(各関数内に入っていく)
    type_check_phase(build_option, &root, &tld_map);

    // STEP5: スタックフレーム割付 && unresolvedな型解決
    allocate_frame_phase(build_option, root.get_mutable_functions());

    // STEP6: コード生成
    if build_option.verbose {
        eprintln!("\tgenerating x64 assembly start.");
    }
    pass::codegen::x64::codegen(build_option, root.give_functions())
}

fn proc_external_module(
    build_option: &option::BuildOption,
    ext_mod: &module::Module,
) -> resource::ASTRoot {
    if ext_mod.is_parent() {
        // サブモジュールをすべて処理して，返す
        let mut all_ast_root: resource::ASTRoot = Default::default();

        for sub in ext_mod.subs.borrow().iter() {
            let sub_root = proc_external_module(build_option, sub);
            all_ast_root.append(sub_root);
        }
        return all_ast_root;
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
) -> resource::ASTRoot {
    let start = time::Instant::now();
    let root = pass::parse(build_option, module_path, tokens);
    let end = time::Instant::now();

    // TODO: パースエラー

    dump_functions_to_stderr(&root, build_option.debug);

    if build_option.verbose {
        eprintln!(
            "    {}: parse {} done in {:?}",
            "STEP2".bold().green(),
            module_path,
            end - start
        );
    }
    root
}

fn resolve_tld_phase(
    build_option: &option::BuildOption,
    root: &resource::ASTRoot,
) -> BTreeMap<String, resource::TopLevelDecl> {
    let func_map = root.get_functions();
    let type_map = root.get_typedefs();

    let function_number = func_map.len() as u64;
    let resolve_tld_pb = indicatif::ProgressBar::new(function_number);
    resolve_tld_pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("#>-"),
    );

    let start = time::Instant::now();

    let mut resolver: resource::TLDResolver = Default::default();

    resolver.resolve_typedefs(build_option, type_map);

    for (func_name, func) in func_map.iter() {
        resolve_tld_pb.set_message(&format!("resolve tld in {}", func_name));

        resolver.resolve_fn(build_option, func_name, func);

        resolve_tld_pb.inc(1);
    }

    let end = time::Instant::now();
    resolve_tld_pb.finish_with_message(&format!("resolve tld done!(in {:?})", end - start));

    resolver.give_map()
}

fn type_check_phase(
    build_option: &option::BuildOption,
    root: &resource::ASTRoot,
    tld_map: &BTreeMap<String, resource::TopLevelDecl>,
) {
    let function_number = root.get_functions().len() as u64;
    let type_check_pb = indicatif::ProgressBar::new(function_number);
    type_check_pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("#>-"),
    );

    let start = time::Instant::now();

    let func_map = root.get_functions();
    for (func_name, func) in func_map.iter() {
        type_check_pb.set_message(&format!("type check in {}", func_name));

        let errors = pass::type_check_fn(build_option, tld_map, func);

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

fn dump_functions_to_stderr(root: &resource::ASTRoot, debug: bool) {
    if !debug {
        return;
    }

    eprintln!("++++++++ dump-functions ++++++++");

    let func_map = root.get_functions();
    for (_name, f) in func_map.iter() {
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
