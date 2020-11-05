extern crate asmpeach;
extern crate clap;
extern crate pld;
extern crate typed_arena;
extern crate yaml_rust;

use arch::x64;
use common::option;
use std::sync::{Arc, Mutex};
use typed_arena::Arena;

mod arch;
mod bundler;
mod common;
mod setup;

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match setup::BUILD_OPTION.matches.subcommand() {
        ("build", Some(_build_m)) => {}
        ("compile", Some(_compile_m)) => {}
        _ => {
            eprintln!("please specify a subcommand. see --help.");
            std::process::exit(1);
        }
    }

    let module_arena = Arena::new();
    let source = setup::BUILD_OPTION.get_source();
    let main_module = bundler::resolve_main(setup::BUILD_OPTION.target, &module_arena, source);

    // ******************
    // *    Compiler    *
    // ******************

    match setup::BUILD_OPTION.target {
        option::Target::X86_64 => x64::main(main_module, &setup::BUILD_OPTION.matches)?,
        option::Target::AARCH64 => unimplemented!(),
    }

    Ok(())
}
