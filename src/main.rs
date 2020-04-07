#[macro_use]
extern crate clap;
extern crate yaml_rust;

use clap::App;

use bundler::bundler_main;
use common::option;

mod bundler;
mod common;
mod compiler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    let (main_fp, build_option) = initialize(matches);

    if build_option.verbose {
        eprintln!("verbose mode is on...");
    }

    // ******************
    // *    Bundler     *
    // ******************

    let main_mod = bundler_main::bundle_main(&build_option, main_fp);

    // ******************
    // *    Compiler    *
    // ******************

    compiler::compile_main(&build_option, main_mod);

    Ok(())
}

fn initialize(matches: clap::ArgMatches) -> (String, option::BuildOption) {
    let d_flag = matches.is_present("debug");
    let v_flag = matches.is_present("verbose");
    let build_option = option::BuildOption::new(d_flag, v_flag);

    (
        matches.value_of("source").unwrap().to_string(),
        build_option,
    )
}
