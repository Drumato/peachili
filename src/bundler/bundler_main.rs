use std::fs;

use crate::bundler::bundle_parser as bp;
use crate::common::{module, operate, option};
use crate::compiler::pass;
use crate::compiler::resource as res;

pub fn bundle_main(build_option: &option::BuildOption, main_fp: String) -> module::Module {
    let main_mod = resolve_dependency(build_option, main_fp, true);
    if build_option.debug {
        eprintln!("++++++++ dump-modules ++++++++");
        eprintln!("{}\n", main_mod);
    }

    main_mod
}

fn resolve_dependency(
    build_option: &option::BuildOption,
    file_path: String,
    is_primary: bool,
) -> module::Module {
    let mut this_mod = if is_primary {
        module::Module::new_primary(file_path)
    } else {
        module::Module::new_external(file_path)
    };

    let tokens = if is_primary {
        let contents = operate::read_program_from_file(&this_mod.file_path);
        Some(pass::tokenize::tokenize(build_option, contents))
    } else {
        this_mod.setup(build_option)
    };

    if tokens.is_none() {
        // ディレクトリだった
        this_mod.parse_directory(build_option);
    } else {
        this_mod.parse_requires(build_option, tokens.unwrap());
    }

    this_mod
}

impl module::Module {
    fn parse_directory(&mut self, build_option: &option::BuildOption) {
        for entry in fs::read_dir(&self.file_path).unwrap() {
            let dir = entry.unwrap();

            let sub_module = resolve_dependency(
                build_option,
                dir.path().to_str().unwrap().to_string(),
                false,
            );
            self.subs.push(sub_module);
        }
    }

    fn parse_requires(&mut self, build_option: &option::BuildOption, tokens: Vec<res::Token>) {
        let mut bundle_parser = bp::BundleParser::new(tokens);

        // `require` 無し -> パース終了
        if !bundle_parser.require_found() {
            return;
        }

        let requires_names: Vec<String> = bundle_parser.parse_each_modules();

        // moveしていい
        for required_name in requires_names {
            let required_module = resolve_dependency(build_option, required_name, false);
            self.requires.push(required_module);
        }

        self.visited = true;
    }

    fn setup(&mut self, build_option: &option::BuildOption) -> Option<Vec<res::Token>> {
        // 相対パス中からチェック
        let metadata = fs::metadata(&self.file_path);

        if metadata.is_ok() {
            return if metadata.unwrap().is_dir() {
                None
            } else {
                let contents = operate::read_program_from_file(&self.file_path);
                Some(pass::tokenize::tokenize(build_option, contents))
            };
        } else {
            // .go をつけて再度検索
            let extended = format!("{}.go", self.file_path);

            let metadata = fs::metadata(&extended);
            if metadata.is_ok() {
                self.file_path = extended;
                let contents = operate::read_program_from_file(&self.file_path);
                return Some(pass::tokenize::tokenize(build_option, contents));
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
        Some(pass::tokenize::tokenize(build_option, contents))
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

fn combined_libpath_and_file(file_path: &String) -> String {
    let lib_path = get_lib_path();
    if lib_path.ends_with("/") {
        format!("{}{}", lib_path, file_path)
    } else {
        format!("{}/{}", lib_path, file_path)
    }
}
