extern crate colored;

use crate::common::{arch, module, option};
use crate::compiler::{general, x64_compiler};

pub fn compile_main(
    build_option: &option::BuildOption,
    main_mod_id: module::ModuleId,
    module_allocator: &module::ModuleAllocator,
) -> arch::x64::AssemblyFile {

    let mut ast_root =
        general::frontend::proc_frontend(build_option, main_mod_id, module_allocator);

    // STEP6: スタックフレーム割付
    x64_compiler::pass::allocate_frame_phase(
        build_option,
        ast_root.get_mutable_functions(),
        module_allocator,
    );

    // STEP7: コード生成
    x64_compiler::pass::codegen::codegen(build_option, ast_root.give_functions(), module_allocator)
}
