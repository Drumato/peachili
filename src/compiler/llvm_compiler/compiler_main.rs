extern crate colored;
extern crate indicatif;

use crate::common::{module, option};
use crate::compiler::general;

pub fn compile_main(
    build_option: &option::BuildOption,
    main_mod_id: module::ModuleId,
    module_allocator: &module::ModuleAllocator,
)
/* -> llvm_scratch::Module */
{
    let main_mod = module_allocator.get_module_ref(&main_mod_id).unwrap();
    let mut _ast_root = general::frontend::proc_frontend(build_option, main_mod, module_allocator);
}
