use crate::common;
use clap::{App, Arg, ArgMatches};

pub const PEACHILI_VERSION: &str = "1.0";

lazy_static! {
    pub static ref BUILD_OPTION: common::option::BuildOption = {
        let matches = create_arg_matches();

        // default_valueがあるので，unwrap()してよい
        let target = common::option::Target::new(matches.value_of("target").unwrap());
        let mut build_option = common::option::BuildOption::new(matches);

        build_option.target = target;

        build_option
    };
}

/// clap::ArgMatches
pub fn create_arg_matches() -> ArgMatches {
    App::new("Peachili - The Peachili Programming Language Driver")
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
                .help("dump IR-Module to hir.dot")
        ])
        .subcommand(App::new("compile")
            .version(PEACHILI_VERSION)
            .author("Drumato <drumato43@gmail.com>"))
        .subcommand(App::new("translate-c")
            .version(PEACHILI_VERSION)
            .author("Drumato <drumato43@gmail.com>"))
        .get_matches()
}
