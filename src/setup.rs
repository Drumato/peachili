use crate::common;
use clap::{App, Arg, ArgMatches};
use id_arena::Arena;
use std::sync::{Arc, Mutex};

pub type ModuleArena = Arc<Mutex<Arena<common::module::Module>>>;
pub type FnArena = Arc<Mutex<Arena<common::ast::Function>>>;
pub type StmtArena = Arc<Mutex<Arena<common::ast::StatementNode>>>;
pub type ExprArena = Arc<Mutex<Arena<common::ast::ExpressionNode>>>;

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

lazy_static! {
    pub static ref MODULE_ARENA: ModuleArena =
        Arc::new(Mutex::new(Arena::new()));
}

lazy_static! {
    pub static ref AST_EXPR_ARENA: ExprArena =
        Arc::new(Mutex::new(Arena::new()));
}

lazy_static! {
    pub static ref AST_STMT_ARENA: StmtArena =
        Arc::new(Mutex::new(Arena::new()));
}

lazy_static! {
    pub static ref AST_FN_ARENA: Arc<Mutex<Arena<common::ast::Function>>> =
        Arc::new(Mutex::new(Arena::new()));
}

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
                .help("x86_64/aarch64"),
        ])
        .subcommand(
            App::new("Compiler")
                .version("1.0")
                .author("Drumato <drumato43@gmail.com>"),
        )
        .get_matches()
}
