extern crate clap;
extern crate typed_arena;
extern crate yaml_rust;
extern crate x64_asm;


use common::{option, module};
use arch::x64;
use typed_arena::Arena;

mod bundler;
mod common;
mod arch;
mod setup;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = setup::create_arg_matches();
    let build_option = initialize(matches);

    let module_arena: Arena<module::ModuleData> = Arena::new();

    // ******************
    // *    Bundler     *
    // ******************

    let source = build_option.get_source();
    let _main_module = bundler::main( build_option.target, source, &module_arena);

    // ******************
    // *    Compiler    *
    // ******************

    match build_option.target {
        option::Target::X86_64 => x64::main()?,
    }

    Ok(())
}

/// コンパイラオプションを定義
/// clap::ArgMatchesを持たせて，いろんなフラグを取得できるようにしておく
fn initialize(matches: clap::ArgMatches) -> option::BuildOption {
    let target = option::Target::new(matches.value_of("target").unwrap());
    let mut build_option = option::BuildOption::new(matches);

    // default_valueがあるので，unwrap()してよい
    build_option.target = target;

    build_option
}
