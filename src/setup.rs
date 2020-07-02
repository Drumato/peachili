use clap::{Arg, App, ArgMatches};


/// clap::ArgMatches
pub fn create_arg_matches() -> ArgMatches {
    App::new("Peachili - The Peachili Programming Language Driver")
        .version("1.0")
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
                .help("x86_64/aarch64")])
        .subcommand(
            App::new("Compiler")
                .version("1.0")
                .author("Drumato <drumato43@gmail.com>")
        ).get_matches()
}