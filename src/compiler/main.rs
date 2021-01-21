use crate::compiler::common::frontend;
use crate::{module, option};

use super::arch::x64;
pub fn compile_main<'a>(
    main_module: module::Module<'a>,
    build_option: option::BuildOption,
) -> Result<String, Box<dyn std::error::Error + 'a>> {
    let target = build_option.target;
    let (ast_roots, raw_type_env) = frontend::main(main_module, build_option)?;

    match target {
        option::Target::X86_64 => {
            let type_env = x64::type_resolve_main(&ast_roots, &raw_type_env)?;
            let lower_ast_roots = ast_roots
                .iter()
                .map(|root| x64::ast_to_lower(root, type_env.clone()).unwrap())
                .collect::<Vec<x64::Root>>();

            let x64_assembly_file = x64::codegen_main(lower_ast_roots);

            Ok(x64_assembly_file)
        }
    }
}
