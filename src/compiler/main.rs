use std::{fs::File, io::prelude::*};

use crate::compiler::common::frontend;
use crate::{module, option};

use super::arch::x64;
pub fn compile_main<'a>(
    alloc: &'a frontend::allocator::Allocator<'a>,
    main_module: module::Module<'a>,
    build_option: option::BuildOption,
) -> Result<(), Box<dyn std::error::Error + 'a>> {
    let target = build_option.target;
    let (ast_roots, raw_type_env) = frontend::main(alloc, main_module, build_option)?;

    match target {
        option::Target::X86_64 => {
            let x64_assembly_file = ast_roots
                .iter()
                .map(|root| {
                    x64::codegen_main(x64::ast_to_lower(root, raw_type_env.clone()).unwrap())
                })
                .collect::<Vec<String>>()
                .join("\n# translation unit's end\n");

            let mut file = File::create("asm.s")?;
            write!(file, "{}", x64_assembly_file)?;
            file.flush()?;
        }
    }

    Ok(())
}
