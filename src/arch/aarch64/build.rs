use crate::arch::aarch64;
use crate::common;
use crate::setup;

/// aarch64アーキテクチャ向けのビルドルーチン
pub fn main(
    module_arena: common::module::ModuleArena,
    main_module_id: common::module::ModuleId,
    matches: &clap::ArgMatches,
) -> Result<(), Box<dyn std::error::Error>> {
    match matches.subcommand() {
        ("compile", Some(compile_m)) => {
            let aarch64_module = aarch64::compile_main(
                module_arena,
                main_module_id,
                compile_m.is_present("verbose-hir"),
                compile_m.is_present("debug"),
            );

            common::file_util::write_program_into("asm.s", aarch64_module.to_assembly());
        }
        _ => eprintln!("please specify a subcommand. see --help."),
    }
    Ok(())
}

/// aarch64用コンパイラのメインルーチン
/// 機械独立なパスを呼び出した後aarch64依存のパスを処理する．
pub fn compile_main(
    module_arena: common::module::ModuleArena,
    main_module_id: common::module::ModuleId,
    verbose_ir: bool,
    debug: bool,
) -> aarch64::ir::Module {
    let (fn_arena, ast_root, type_env, stack_frame) =
        common::pass::frontend(module_arena, main_module_id, debug);
    let (ir_module, _local_cfg) =
        common::pass::backend(fn_arena, ast_root, &type_env, setup::BUILD_OPTION.target, verbose_ir);

    aarch64::pass::codegen_main(ir_module, stack_frame)
}
