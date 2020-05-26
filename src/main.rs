#[macro_use]
extern crate clap;
extern crate typed_arena;
extern crate yaml_rust;

use clap::App;
use typed_arena::Arena;

use bundler::bundler_main;
use common::option;

pub mod assembler;
pub mod bundler;
pub mod common;
pub mod compiler;
pub mod llvm_main;
pub mod x64_main;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arena: Arena<common::module::Module> = Arena::new();

    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    let (main_file_path, build_option) = initialize(matches);

    if build_option.verbose {
        eprintln!("verbose mode is on...");
    }

    // ******************
    // *    Bundler     *
    // ******************

    let main_mod = bundler_main::bundle_main(&build_option, main_file_path, &arena);

    // ******************
    // *    Compiler    *
    // ******************

    match build_option.target {
        option::Target::X86_64 => x64_main::main(&build_option, main_mod)?,
        option::Target::LLVMIR => llvm_main::main(&build_option, main_mod)?,
    }

    Ok(())
}

fn initialize(matches: clap::ArgMatches) -> (String, option::BuildOption) {
    let mut build_option: option::BuildOption = Default::default();
    build_option.debug = matches.is_present("debug");
    build_option.verbose = matches.is_present("verbose");
    build_option.stop_assemble = matches.is_present("stop-assemble");
    build_option.stop_link = matches.is_present("stop-link");

    let lang_str = std::env::var("LANG").unwrap();
    let lang = option::Language::new(lang_str);
    if let Some(target_str) = matches.value_of("target") {
        build_option.target = option::Target::new(target_str);
    }

    build_option.language = lang;

    (
        matches.value_of("source").unwrap().to_string(),
        build_option,
    )
}
