#[macro_use]
extern crate clap;
extern crate yaml_rust;

use clap::App;

use common::{module, option};

mod common;
mod compiler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    // TODO: --llvm-ir フラグが指定されたとき, 別にinitialize関数を定義する．
    // Module構造体にわたす型パラメータが異なるものになるため．

    let (mut main_mod, build_options) = initialize(matches);

    if build_options.verbose {
        eprintln!("verbose mode is on...");
    }

    // ******************
    // *    Bundler     *
    // ******************

    // ******************
    // *    Compiler    *
    // ******************

    let main_mod = compiler::proc_frontend(&build_options, main_mod);
    Ok(())
}

fn initialize(
    matches: clap::ArgMatches,
) -> (
    module::Module<compiler::resource::PFunction>,
    option::BuildOption,
) {
    let d_flag = matches.is_present("debug");
    let v_flag = matches.is_present("verbose");
    let main_path = matches.value_of("source").unwrap();

    let build_option = option::BuildOption::new(d_flag, v_flag);

    let main_module = module::Module::new_primary(main_path.to_string());
    (main_module, build_option)
}
