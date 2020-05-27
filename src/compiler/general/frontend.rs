use crate::common::{error, module, option};
use crate::compiler::general::{pass, resource as res};

pub fn proc_frontend(
    build_option: &option::BuildOption,
    main_mod: &module::Module,
    module_allocator: &module::ModuleAllocator,
) -> res::ASTRoot {
    process_main_module(build_option, main_mod, module_allocator)
}

fn process_main_module(
    build_option: &option::BuildOption,
    main_mod: &module::Module,
    module_allocator: &module::ModuleAllocator,
) -> res::ASTRoot {
    // 字句解析はBundlerがすでに終わらせている
    let tokens = main_mod.tokens.clone();

    // STEP2: parse
    let mut root = pass::parse_phase(build_option, &main_mod.file_path, tokens);

    // mainモジュールが依存するモジュールすべてのパースを行い，全体のASTRootを構築する
    let requires = main_mod.get_locked_requires();
    for req_mod_id in requires.iter() {
        let req_mod = module_allocator.get_module_ref(req_mod_id).unwrap();
        let req_root = proc_external_module(build_option, req_mod, module_allocator);
        root.append(req_root);
    }

    // STEP3: resolve TLD
    // TODO: see Issue#12
    let tld_map = pass::resolve_tld_phase(build_option, &root);

    if !tld_map.contains_key("main") {
        error::CompileError::main_must_exist().emit_stderr(&main_mod.file_path, build_option);

        std::process::exit(1);
    }

    // STEP4: unresolved な型解決
    pass::resolve_unknown_type_phase(build_option, root.get_mutable_functions(), &tld_map);

    // STEP5: 型検査(各関数内に入っていく)
    pass::type_check_phase(build_option, &root, &tld_map);

    root
}

fn proc_external_module(
    build_option: &option::BuildOption,
    ext_mod: &module::Module,
    module_allocator: &module::ModuleAllocator,
) -> res::ASTRoot {
    // ext_mod がディレクトリの場合
    if ext_mod.is_parent() {
        // サブモジュールをすべて処理して，返す
        let mut all_ast_root: res::ASTRoot = Default::default();

        let submodules = ext_mod.get_locked_submodules();
        for sub_id in submodules.iter() {
            let sub = module_allocator.get_module_ref(sub_id).unwrap();
            let sub_root = proc_external_module(build_option, sub, module_allocator);
            all_ast_root.append(sub_root);
        }

        return all_ast_root;
    }

    // 字句解析はBundlerがすでに終わらせている
    let tokens = ext_mod.tokens.clone();

    // STEP2: parse
    pass::parse_phase(build_option, &ext_mod.file_path, tokens)
}
