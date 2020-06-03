extern crate colored;
extern crate indicatif;

use crate::common::{module, option};
use crate::compiler::{general, llvm_compiler};

pub fn compile_main(
    build_option: &option::BuildOption,
    main_mod_id: module::ModuleId,
    module_allocator: &module::ModuleAllocator,
) -> llvm_scratch::core::module::Module {
    let ast_root = general::frontend::proc_frontend(build_option, main_mod_id, module_allocator);

    llvm_compiler::pass::codegen(build_option, ast_root, module_allocator)
}
