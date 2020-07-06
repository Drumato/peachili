extern crate clap;
extern crate id_arena;
extern crate x64_asm;
extern crate yaml_rust;

use arch::x64;
use common::option;

mod arch;
mod bundler;
mod common;
mod setup;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ******************
    // *    Bundler     *
    // ******************

    let source = setup::BUILD_OPTION.get_source();
    let main_module = bundler::resolve_main(setup::MODULE_ARENA.clone(), source);

    // ******************
    // *    Compiler    *
    // ******************

    match setup::BUILD_OPTION.target {
        option::Target::X86_64 => x64::main(main_module)?,
    }

    Ok(())
}
