extern crate colored;
extern crate indicatif;

use crate::common::{arch, module, operate, option};
use crate::compiler::{pass, resource};

pub fn compile_main(
    build_option: &option::BuildOption,
    main_mod: module::Module,
) -> Result<arch::x64::AssemblyFile, Box<dyn std::error::Error>> {
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
    let tokens = pass::tokenize_phase(build_option, &main_mod.file_path, contents);

    // STEP2: parse
    let mut root = pass::parse_phase(build_option, &main_mod.file_path, tokens);

    for req_mod in main_mod.requires.borrow().iter() {
        let req_root = proc_external_module(build_option, req_mod);
        root.append(req_root);
    }

    // STEP3: resolve TLD
    // TODO: see Issue#12
    let tld_map = pass::resolve_tld_phase(build_option, &root);

    // STEP4: unresolved な型解決
    pass::resolve_unknown_type_phase(build_option, root.get_mutable_functions(), &tld_map);

    // STEP5: 型検査(各関数内に入っていく)
    pass::type_check_phase(build_option, &root, &tld_map);

    // STEP6: スタックフレーム割付
    pass::allocate_frame_phase(build_option, root.get_mutable_functions());

    // STEP7: コード生成
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
    let tokens = pass::tokenize_phase(build_option, &ext_mod.file_path, contents);

    // STEP2: parse
    pass::parse_phase(build_option, &ext_mod.file_path, tokens)
}
