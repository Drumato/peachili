#[macro_use]
extern crate clap;
extern crate typed_arena;
extern crate yaml_rust;

use std::io::Write;

use clap::App;
use typed_arena::Arena;

use bundler::bundler_main;
use common::option;

pub mod assembler;
pub mod bundler;
pub mod common;
pub mod compiler;

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

    let assembly_file = compiler::compile_main(&build_option, main_mod)?;

    if build_option.stop_assemble {
        // アセンブリファイルを生成してプロセスを終了
        // とりあえずAT&T syntaxで
        let mut asm_output = std::fs::File::create(&assembly_file.file_path).unwrap();
        asm_output.write_all(assembly_file.to_at_code().as_bytes())?;
        std::process::exit(0);
    }

    // *****************
    // *   Assembler   *
    // *****************
    let elf_builder = assembler::x64_assemble(&build_option, assembly_file);

    if build_option.stop_link {
        // オブジェクトファイルを生成して終了
        elf_builder.generate_elf_file("obj.o");
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

    build_option.language = lang;

    (
        matches.value_of("source").unwrap().to_string(),
        build_option,
    )
}
