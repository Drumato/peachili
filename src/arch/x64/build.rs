use crate::arch::x64;
use crate::common;
use crate::setup;

/// x64アーキテクチャ向けのビルドルーチン
pub fn main(
    module_arena: common::module::ModuleArena,
    main_module_id: common::module::ModuleId,
    matches: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        ("compile", Some(compile_m)) => {
            let x64_module = compile_main(
                module_arena,
                main_module_id,
                compile_m.is_present("verbose-hir"),
                compile_m.is_present("debug"),
            );

            common::file_util::write_program_into("asm.s", x64_module.to_atandt());
        }
        _ => eprintln!("please specify a subcommand. see --help."),
    }
    Ok(())
}

/// x64用コンパイラのメインルーチン
/// 機械独立なパスを呼び出した後x64依存のパスを処理する．
pub fn compile_main(
    module_arena: common::module::ModuleArena,
    main_module_id: common::module::ModuleId,
    verbose_ir: bool,
    debug: bool,
) -> x64::ir::Module {
    let (fn_arena, ast_root, type_env, stack_frame) =
        common::pass::frontend(module_arena, main_module_id, debug);
    let (ir_module, _local_cfg) = common::pass::backend(
        fn_arena,
        ast_root,
        &type_env,
        setup::BUILD_OPTION.target,
        verbose_ir,
    );

    x64::pass::codegen_main(ir_module, stack_frame)
}
