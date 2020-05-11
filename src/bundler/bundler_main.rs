extern crate typed_arena;

use std::fs;

use typed_arena::Arena;

use crate::bundler::bundle_parser as bp;
use crate::common::{error, module, operate, option};
use crate::compiler::pass;
use crate::compiler::resource as res;

pub fn bundle_main<'b>(
    build_option: &option::BuildOption,
    main_fp: String,
    arena: &'b Arena<module::Module<'b>>,
) -> module::Module<'b> {
    let mut main_mod = module::Module::new_primary(main_fp);

    let main_tokens = main_mod.setup(build_option);

    if main_tokens.is_none() {
        panic!(
            "main module must be a peachili file, got directory -> {}",
            main_mod.file_path
        );
    }

    main_mod.parse_requires(build_option, main_tokens.unwrap(), arena);

    if build_option.debug {
        eprintln!("++++++++ dump-modules ++++++++");
        eprintln!("{}\n", main_mod);
    }

    main_mod
}

#[allow(clippy::unnecessary_unwrap)]
fn resolve_dependency<'b>(
    build_option: &option::BuildOption,
    file_path: String,
    arena: &'b Arena<module::Module<'b>>,
) -> module::Module<'b> {
    let mut this_mod = module::Module::new_external(file_path);

    let tokens = this_mod.setup(build_option);

    if tokens.is_none() {
        // ディレクトリだった
        this_mod.parse_directory(build_option, arena);
    } else {
        this_mod.parse_requires(build_option, tokens.unwrap(), arena);
    }

    this_mod
}

impl<'a> module::Module<'a> {
    fn parse_directory(
        &mut self,
        build_option: &option::BuildOption,
        arena: &'a Arena<module::Module<'a>>,
    ) {
        for entry in fs::read_dir(&self.file_path).unwrap() {
            let dir = entry.unwrap();

            let sub_module = arena.alloc(resolve_dependency(
                build_option,
                dir.path().to_str().unwrap().to_string(),
                arena,
            ));
            self.subs.borrow_mut().push(sub_module);
        }
    }

    fn parse_requires(
        &mut self,
        build_option: &option::BuildOption,
        tokens: Vec<res::Token>,
        arena: &'a Arena<module::Module<'a>>,
    ) {
        let mut bundle_parser = bp::BundleParser::new(tokens);

        // `require` 無し -> パース終了
        if !bundle_parser.require_found() {
            return;
        }

        let requires_names: Vec<String> = bundle_parser.parse_each_modules();

        // moveしていい
        for required_name in requires_names {
            let required_module =
                arena.alloc(resolve_dependency(build_option, required_name, arena));
            self.requires.borrow_mut().push(required_module);
        }

        self.visited = true;
    }

    #[allow(clippy::unnecessary_unwrap)]
    fn setup(&mut self, build_option: &option::BuildOption) -> Option<Vec<res::Token>> {
        // 相対パス中からチェック
        let metadata = fs::metadata(&self.file_path);

        if metadata.is_ok() {
            return if metadata.unwrap().is_dir() {
                None
            } else {
                let contents = operate::read_program_from_file(&self.file_path);
                let (tokens, errors) = pass::tokenize(build_option, contents);

                if !errors.is_empty() {
                    emit_all_errors_and_exit(&errors, &self.file_path);
                }

                Some(tokens)
            };
        } else {
            // .go をつけて再度検索
            let extended = format!("{}.go", self.file_path);

            let metadata = fs::metadata(&extended);
            if metadata.is_ok() {
                self.file_path = extended;
                let contents = operate::read_program_from_file(&self.file_path);

                let (tokens, errors) = pass::tokenize(build_option, contents);

                if !errors.is_empty() {
                    emit_all_errors_and_exit(&errors, &self.file_path);
                }

                return Some(tokens);
            }
        }

        // 環境変数からチェック
        let combined_path = combined_libpath_and_file(&self.file_path);

        let metadata = fs::metadata(&combined_path);

        if metadata.is_ok() {
            self.file_path = combined_path;
            // .go をつけていないので，必ずディレクトリ
            return None;
        }

        // .go をつけて再度チェック

        let extended = format!("{}.go", combined_path);
        let metadata = fs::metadata(&extended);

        if metadata.is_err() {
            // エラー
            panic!("not found such a module -> {}", combined_path);
        }

        self.file_path = combined_path;

        let contents = operate::read_program_from_file(&self.file_path);
        let (tokens, errors) = pass::tokenize(build_option, contents);

        if !errors.is_empty() {
            emit_all_errors_and_exit(&errors, &self.file_path);
        }

        Some(tokens)
    }
}

fn get_lib_path() -> String {
    use std::env;

    let cur_lib_path = env::var("PEACHILI_LIB_PATH");

    if cur_lib_path.is_err() {
        panic!("`PEACHILI_LIB_PATH` was not found.");
    }

    cur_lib_path.unwrap()
}

fn combined_libpath_and_file(file_path: &str) -> String {
    let lib_path = get_lib_path();
    if lib_path.ends_with('/') {
        format!("{}{}", lib_path, file_path)
    } else {
        format!("{}/{}", lib_path, file_path)
    }
}

fn emit_all_errors_and_exit(errors: &[error::CompileError], module_path: &str) -> ! {
    for err in errors.iter() {
        err.emit_stderr(module_path);
    }

    std::process::exit(1);
}
