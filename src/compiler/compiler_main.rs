use std::fs;

use crate::common::{module, option};
use crate::compiler::{pass, resource};

pub fn proc_frontend(
    build_option: &option::BuildOption,
    this_mod: module::Module,
) -> module::Module {
    let contents = read_program_from_file(&this_mod.file_path);
    let tokens = pass::tokenize::tokenize(build_option, contents);

    dump_tokens_to_stderr(&tokens, build_option.debug);

    let (this_mod, functions) = pass::parse::parse(build_option, tokens, this_mod);

    dump_functions_to_stderr(&functions, build_option.debug);
    this_mod
}

fn dump_tokens_to_stderr(tokens: &[resource::Token], debug: bool) {
    if !debug {
        return;
    }

    for t in tokens.iter() {
        eprintln!("\t{}", t);
    }
}

fn dump_functions_to_stderr(functions: &[resource::PFunction], debug: bool) {
    if !debug {
        return;
    }

    for f in functions.iter() {
        eprintln!("{}", f);
    }
}

fn read_program_from_file(path: &str) -> String {
    let result_contents = fs::read_to_string(path);

    if result_contents.is_err() {
        panic!("read {} failed.", path);
    }

    result_contents.unwrap()
}
