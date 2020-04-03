use std::fs;

use crate::common::{module, option};
use crate::compiler::pass;

pub fn proc_frontend(
    build_option: &option::BuildOption,
    this_mod: module::Module,
) -> module::Module {
    let contents = read_program_from_file(&this_mod.file_path);
    let tokens = pass::tokenize::tokenize(build_option, contents);

    if build_option.debug {
        for t in tokens.iter() {
            eprintln!("\t{}", t.to_string());
        }
    }

    this_mod
}

fn read_program_from_file(path: &str) -> String {
    let result_contents = fs::read_to_string(path);

    if result_contents.is_err() {
        panic!("read {} failed.", path);
    }

    result_contents.unwrap()
}
