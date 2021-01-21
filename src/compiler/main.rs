use crate::compiler::common::frontend;
use crate::{module, option};

use super::arch::{aarch64, x86_64};
pub fn compile_main<'a>(
    main_module: module::Module<'a>,
    build_option: option::BuildOption,
) -> Result<String, Box<dyn std::error::Error + 'a>> {
    let target = build_option.target;
    let (ast_roots, raw_type_env) = frontend::main(main_module, build_option)?;

    match target {
        option::Target::X86_64 => {
            let type_env = x86_64::type_resolve_main(&ast_roots, &raw_type_env)?;
            let lower_ast_roots = ast_roots
                .iter()
                .map(|root| x86_64::ast_to_lower(root, type_env.clone()).unwrap())
                .collect::<Vec<x86_64::Root>>();

            let assembly_file = x86_64::codegen_main(lower_ast_roots);

            Ok(assembly_file)
        }
        option::Target::AArch64 => {
            let type_env = aarch64::type_resolve_main(&ast_roots, &raw_type_env)?;
            let lower_ast_roots = ast_roots
                .iter()
                .map(|root| aarch64::ast_to_lower(root, type_env.clone()).unwrap())
                .collect::<Vec<aarch64::Root>>();

            let assembly_file = aarch64::codegen_main(lower_ast_roots);

            Ok(assembly_file)
        }
    }
}
