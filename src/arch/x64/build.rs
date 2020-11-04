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
        ("build", Some(build_m)) => {
            let link_option = pld::LinkOption {
                entry_point: "startup::initialize".to_string(),
            };
            compile_main(
                module_arena,
                main_module_id,
                build_m.is_present("verbose-hir"),
                build_m.is_present("debug"),
                link_option.entry_point.to_string(),
            );
        }
        ("compile", Some(compile_m)) => {
            compile_main(
                module_arena,
                main_module_id,
                compile_m.is_present("verbose-hir"),
                compile_m.is_present("debug"),
                String::new(),
            );
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
    _verbose_ir: bool,
    debug: bool,
    _entry_point: String,
) -> () {
    let _ast_root = common::pass::frontend(module_arena, main_module_id, debug);
}
