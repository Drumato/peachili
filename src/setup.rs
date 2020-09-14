use crate::common;
use clap::{App, Arg, ArgMatches};

pub const PEACHILI_VERSION: &str = "1.0";

lazy_static! {
    pub static ref BUILD_OPTION: common::option::BuildOption = {
        let matches = create_arg_matches();

        // default_valueがあるので，unwrap()してよい
        let target = match matches.subcommand() {
            ("build", Some(build_m)) => Some(build_m.value_of("target").unwrap()),
            ("compile", Some(compile_m)) => Some(compile_m.value_of("target").unwrap()),
            _ => None,
        };

        let mut build_option = common::option::BuildOption::new(matches.clone());

        if let Some(target) = target {
            build_option.target = common::option::Target::new(target);
        }

        build_option
    };
}

/// clap::ArgMatches
fn create_arg_matches() -> ArgMatches {
    App::new("Peachili - The Peachili Programming Language Driver")
        .version(PEACHILI_VERSION)
        .author("Drumato <drumato43@gmail.com>")
        .subcommand(
            App::new("compile")
                .version(PEACHILI_VERSION)
                .author("Drumato <drumato43@gmail.com>")
                .args(&[
                    // コンパイル対象のファイル
                    Arg::with_name("source")
                        .required(true)
                        .index(1)
                        .help("Sets the input file to use"),
                    // 生成するコードの対象
                    Arg::with_name("target")
                        .default_value("x86_64")
                        .short('t')
                        .long("target")
                        .help("x86_64/aarch64"),
                    // IRのダンプ
                    Arg::with_name("verbose-hir")
                        .long("verbose-hir")
                        .help("dump IR-Module to hir.dot"),
                    // debugオプション
                    Arg::with_name("debug").long("debug").help("debug"),
                ]),
        )
        .subcommand(
            App::new("build")
                .version(PEACHILI_VERSION)
                .author("Drumato <drumato43@gmail.com>")
                .args(&[
                    // コンパイル対象のファイル
                    Arg::with_name("source")
                        .required(true)
                        .index(1)
                        .help("Sets the input file to use"),
                    // 生成するコードの対象
                    Arg::with_name("target")
                        .default_value("x86_64")
                        .short('t')
                        .long("target")
                        .help("x86_64/aarch64"),
                    // IRのダンプ
                    Arg::with_name("verbose-hir")
                        .long("verbose-hir")
                        .help("dump IR-Module to hir.dot"),
                    // debugオプション
                    Arg::with_name("debug").long("debug").help("debug"),
                ]),
        )
        .get_matches()
}
