use crate::compiler::common::frontend;
use crate::{module, option};
pub fn compile_main<'a>(main_module: module::Module<'a>, build_option: option::BuildOption) {
    match build_option.target {
        option::Target::X86_64 => {
            let _root = frontend::main(main_module);
        }
    }
}
