extern crate asmpeach;
extern crate pld;
extern crate typed_arena;

use structopt::StructOpt;
use typed_arena::Arena;

pub mod bundler;
pub mod compiler;
pub mod module;
pub mod option;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let module_arena = Arena::new();
    let alloc = Default::default();
    let build_option = option::BuildOption::from_args();

    // ******************
    // *    Compiler    *
    // ******************

    match build_option.cmd {
        option::Command::Build => unimplemented!(),
        option::Command::Compile { ref source_file } => {
            let main_module =
                bundler::resolve_main(build_option.target, &module_arena, source_file.clone());
            match compiler::compile_main(&alloc, main_module, build_option) {
                Ok(_) => {}
                Err(e) => eprintln!("{}", e),
            }
        }
    }

    Ok(())
}
