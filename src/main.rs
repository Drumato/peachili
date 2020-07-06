extern crate clap;
extern crate id_arena;
extern crate yaml_rust;
extern crate x64_asm;

use common::{option};
use arch::x64;

mod bundler;
mod common;
mod arch;
mod setup;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // ******************
    // *    Bundler     *
    // ******************

    let source = setup::BUILD_OPTION.get_source();
    let main_module = bundler::main(source);

    // ******************
    // *    Compiler    *
    // ******************

    match setup::BUILD_OPTION.target {
        option::Target::X86_64 => x64::main(main_module)?,
    }

    Ok(())
}
