use crate::common;

/// x64アーキテクチャ向けのビルドルーチン
pub fn main<'a>(
    main_module: common::module::Module<'a>,
    build_option: common::option::BuildOption,
) -> Result<(), Box<dyn std::error::Error>> {
    match build_option.cmd {
        common::option::Command::Build => {
            let link_option = pld::LinkOption {
                entry_point: "startup::initialize".to_string(),
            };
            compile_main(
                main_module,
                link_option.entry_point.to_string(),
            );
        }
        common::option::Command::Compile => {
            compile_main(
                main_module,
                String::new(),
            );
        }
    }
    Ok(())
}

/// x64用コンパイラのメインルーチン
/// 機械独立なパスを呼び出した後x64依存のパスを処理する．
pub fn compile_main<'a>(
    main_module: common::module::Module<'a>,
    _entry_point: String,
) -> () {
    let _ast_root = common::pass::frontend(main_module);
}
