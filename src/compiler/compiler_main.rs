use std::fs;

use crate::common::{module, option};
use crate::compiler::{pass, resource};

pub fn proc_frontend(
    build_option: &option::BuildOption,
    mut this_mod: module::Module<resource::PFunction>,
) -> module::Module<resource::PFunction> {
    let contents = read_program_from_file(&this_mod.file_path);
    let tokens = pass::tokenize::tokenize(build_option, contents);

    dump_tokens_to_stderr(&tokens, build_option.debug);

    let (this_mod, functions) = pass::parse::parse(build_option, tokens, this_mod);
    this_mod
}

fn dump_tokens_to_stderr(tokens: &[resource::Token], debug: bool) {
    if debug {
        for t in tokens.iter() {
            eprintln!("\t{}", t.to_string());
        }
    }
}

fn read_program_from_file(path: &str) -> String {
    let result_contents = fs::read_to_string(path);

    if result_contents.is_err() {
        panic!("read {} failed.", path);
    }

    result_contents.unwrap()
}
