extern crate asmpeach;
extern crate pld;
extern crate typed_arena;

use structopt::StructOpt;
use arch::x64;
use common::option;
use typed_arena::Arena;

mod arch;
mod bundler;
mod common;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let module_arena = Arena::new();
    let build_option = option::BuildOption::from_args();
    let main_module = bundler::resolve_main(build_option.target, &module_arena, build_option.source_file.clone());

    // ******************
    // *    Compiler    *
    // ******************

    match build_option.target {
        option::Target::X86_64 => x64::main(main_module, build_option)?,
    }

    Ok(())
}
