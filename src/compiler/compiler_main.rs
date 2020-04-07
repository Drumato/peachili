use crate::common::{module, operate, option};
use crate::compiler::{pass, resource};

pub fn compile_main(build_option: &option::BuildOption, main_mod: module::Module) {
    // TODO: 所有権を考えつつ，どうやってrequires を処理するか．
    // 今は適当にmainだけやっとく

    let contents = operate::read_program_from_file(&main_mod.file_path);
    let tokens = pass::tokenize::tokenize(build_option, contents);

    dump_tokens_to_stderr(&tokens, build_option.debug);

    let (_main_mod, functions) = pass::parse::parse(build_option, tokens, main_mod);

    dump_functions_to_stderr(&functions, build_option.debug);
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
