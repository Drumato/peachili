extern crate clap;
extern crate id_arena;
extern crate x64_asm;
extern crate yaml_rust;

use arch::x64;
use common::option;
use id_arena::Arena;
use std::sync::{Arc, Mutex};

mod arch;
mod bundler;
mod common;
mod debug;
mod setup;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let module_arena: common::module::ModuleArena = Arc::new(Mutex::new(Arena::new()));
    let source = setup::BUILD_OPTION.get_source();
    let main_module = bundler::resolve_main(module_arena.clone(), source);

    // ******************
    // *    Compiler    *
    // ******************

    match setup::BUILD_OPTION.target {
        option::Target::X86_64 => x64::main(
            module_arena,
            main_module,
            setup::BUILD_OPTION.matches.is_present("verbose-hir"),
            setup::BUILD_OPTION.matches.is_present("debug"),
        )?,
    }

    Ok(())
}
