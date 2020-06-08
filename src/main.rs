#[macro_use]
extern crate clap;
extern crate id_arena;
extern crate yaml_rust;

use std::sync::{Arc, Mutex};

use clap::App;

use bundler::bundler_main;
use common::{module, option};
use compiler::general::resource as res;

pub mod assembler;
pub mod bundler;
pub mod common;
pub mod compiler;
pub mod llvm_main;
pub mod x64_main;
pub mod linker;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // スタティックなライフタイムを必要とするアロケータ達
    let mut module_allocator: module::ModuleAllocator = Default::default();
    let const_pool: res::ConstAllocator = Default::default();

    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    let (main_file_path, build_option) = initialize(matches);

    // ******************
    // *    Bundler     *
    // ******************

    // 各モジュールで共有したいので，Arc<Mutex<T>>に
    let main_mod_id = bundler_main::bundle_main(
        &build_option,
        main_file_path,
        &mut module_allocator,
        Arc::new(Mutex::new(const_pool)),
    );

    // ******************
    // *    Compiler    *
    // ******************

    match build_option.target {
        option::Target::X86_64 => x64_main::main(&build_option, main_mod_id, module_allocator)?,
        option::Target::LLVMIR => llvm_main::main(&build_option, main_mod_id, module_allocator)?,
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
    if let Some(arch_str) = matches.value_of("arch") {
        build_option.arch = option::Architecture::new(arch_str);
    }

    build_option.language = lang;

    (
        matches.value_of("source").unwrap().to_string(),
        build_option,
    )
}
