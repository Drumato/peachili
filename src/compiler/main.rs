use crate::compiler::common::frontend;
use crate::{module, option};

use super::arch::{aarch64, x86_64};
pub fn compile_main<'a>(
    main_module: module::Module<'a>,
    build_option: option::BuildOption,
) -> Result<String, Box<dyn std::error::Error + 'a>> {
    let target = build_option.target;
    let ast_roots = frontend::main(main_module, build_option)?;

    match target {
        option::Target::X86_64 => {
            let assembly_file = x86_64::codegen_main(ast_roots);

            Ok(assembly_file)
        }
        option::Target::AArch64 => {
            let assembly_file = aarch64::codegen_main(ast_roots);

            Ok(assembly_file)
        }
    }
}
