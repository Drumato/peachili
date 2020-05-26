extern crate colored;
extern crate indicatif;

use crate::common::{module, option};
use crate::compiler::general;

pub fn compile_main(build_option: &option::BuildOption, main_mod: module::Module)
/* -> llvm_scratch::Module */
{
    let mut _ast_root = general::frontend::proc_frontend(build_option, &main_mod);
}
