use crate::common::{
    error::{BundleErrorKind as BEK, CompileError as CE},
    file_util as fu, module as m, option as opt,
};
use crate::setup;
use id_arena::Arena;
use std::sync::{Arc, Mutex, MutexGuard};

use std::fs;

pub fn resolve_main(arena: Arc<Mutex<Arena<m::Module>>>, source_name: String) -> m::ModuleId {
    let file_contents = try_to_get_file_contents(&source_name);
    let main_mod = alloc_main_module(arena.lock().unwrap(), source_name);

    // スタートアップ･ライブラリの追加
    let startup_module_path = setup_startup_routine();
    process_ext_module(arena.clone(), startup_module_path, "startup".to_string());

    // mainが参照するモジュールに対しそれぞれprocess_ext_moduleする
    let main_requires = collect_import_modules_from_program(file_contents);

    add_dependencies_to(arena, main_mod, main_requires, false);

    main_mod
}

/// (間接的にではあるが)再帰的に呼び出される
fn process_ext_module(
    arena: Arc<Mutex<Arena<m::Module>>>,
    ext_fp: String,
    ext_name: String,
) -> m::ModuleId {
    // 相対パス部分と拡張子を削る
    let ext_name = if ext_name.contains('/') {
        let ext_name = ext_name.split('/').collect::<Vec<&str>>().pop().unwrap();
        if ext_name.contains('.') {
            ext_name.split('.').collect::<Vec<&str>>()[0].to_string()
        } else {
            ext_name.to_string()
        }
    } else {
        ext_name
    };
    let parent_mod = alloc_ext_module(arena.lock().unwrap(), ext_fp.to_string(), ext_name);

    // TODO: エラー出したほうがいいかも
    let ext_module_is_dir = fs::metadata(&ext_fp).unwrap().is_dir();

    if ext_module_is_dir {
        // parent_modのsubsにモジュールをぶら下げる
        process_submodules(arena, &parent_mod);
    } else {
        // 普通のファイルと同じように処理する
        let file_contents = try_to_get_file_contents(&ext_fp);
        let requires = collect_import_modules_from_program(file_contents);

        add_dependencies_to(arena, parent_mod, requires, false);
    }

    parent_mod
}

/// ディレクトリ内の各ファイルに対して，resolveを実行する
fn process_submodules(arena: Arc<Mutex<Arena<m::Module>>>, dir_module_id: &m::ModuleId) {
    let parent_module_path = arena
        .lock()
        .unwrap()
        .get_mut(*dir_module_id)
        .unwrap()
        .copy_path();

    for entry in fs::read_dir(&parent_module_path).unwrap() {
        let file_in_dir = entry.unwrap();
        let child_module_name = file_in_dir.path().to_str().unwrap().to_string();
        let resolved_path = resolve_path_from_name(child_module_name.to_string());

        let child_module = process_ext_module(arena.clone(), resolved_path, child_module_name);
        arena
            .lock()
            .unwrap()
            .get_mut(*dir_module_id)
            .unwrap()
            .add_child_module(child_module);
    }
}

/// 依存ノードを追加する
fn add_dependencies_to(
    arena: Arc<Mutex<Arena<m::Module>>>,
    src_mod_id: m::ModuleId,
    dependencies: Vec<String>,
    is_dir: bool,
) {
    for req in dependencies {
        let req_path = resolve_path_from_name(req.to_string());
        let ext_mod = process_ext_module(arena.clone(), req_path, req);

        if is_dir {
            arena
                .lock()
                .unwrap()
                .get_mut(src_mod_id)
                .unwrap()
                .add_child_module(ext_mod);
        } else {
            arena
                .lock()
                .unwrap()
                .get_mut(src_mod_id)
                .unwrap()
                .add_reference_module(ext_mod);
        }
    }
}

/// モジュール名から，モジュールが存在する絶対パスを取得する．
/// 相対パスに存在するか，PEACHILI_LIB_PATH/lib/に存在すればOK
/// そうでなければとりあえずpanic!する
fn resolve_path_from_name(module_name: String) -> String {
    // 普通に相対パスで検索
    let resolved_path = search_module(module_name.to_string());
    if let Some(relative_path) = resolved_path {
        return relative_path;
    }

    // PEACHILI_LIB_PATH/lib をつけて検索
    let resolved_path = search_module(format!("{}{}", get_lib_path(), module_name));
    if let Some(lib_path) = resolved_path {
        return lib_path;
    }

    panic!("not found such a module -> `{}`", module_name)
}

/// モジュールが存在するかチェック．
/// DIR_MODULEの可能性を考えて，`.go`無しとありの2パターンで検索する
fn search_module(module_name: String) -> Option<String> {
    let resolved_dir = search_directory(module_name.to_string());

    if let Some(dir_path) = resolved_dir {
        return Some(dir_path);
    }

    // ディレクトリがなかったので，拡張子をつけて再度チェック
    let resolved_file = search_peachili_program(format!("{}.go", module_name));
    if let Some(file_path) = resolved_file {
        return Some(file_path);
    }

    None
}

/// 引数に渡したディレクトリが存在するかチェック
fn search_peachili_program(file_name: String) -> Option<String> {
    let metadata = fs::metadata(file_name.to_string());

    // そもそもファイルが存在しなかった
    if metadata.is_err() {
        return None;
    }

    // 拡張子をつけてファイルを見つけられた -> ソースファイルを発見した
    Some(file_name)
}

/// 引数に渡したPeachiliファイルが存在するかチェック
fn search_directory(dir_name: String) -> Option<String> {
    let metadata = fs::metadata(dir_name.to_string());

    // そもそもファイルが存在しなかった
    if metadata.is_err() {
        return None;
    }

    // 何も拡張子をつけないでファイルを見つけられた -> ディレクトリを発見した
    Some(dir_name)
}

/// PRIMARYなモジュールをアロケートして返す
fn alloc_main_module(mut arena: MutexGuard<Arena<m::Module>>, main_fp: String) -> m::ModuleId {
    arena.alloc(m::Module::new_primary(main_fp, "main".to_string()))
}

fn alloc_ext_module(
    mut arena: MutexGuard<Arena<m::Module>>,
    ext_fp: String,
    ext_name: String,
) -> m::ModuleId {
    arena.alloc(m::Module::new_external(ext_fp, ext_name))
}

/// ファイル先頭にある任意数の `import <module-name>;` を解読して返す
fn collect_import_modules_from_program(file_contents: String) -> Vec<String> {
    let mut requires = Vec::new();
    let lines_iter = file_contents.lines();

    for l in lines_iter {
        // とりあえずTopLevelDeclがくるまでループしておく
        if l.contains("func") || l.contains("pubtype") || l.contains("struct") {
            return requires;
        }

        // importがなければ空行
        if !l.contains("import") {
            continue;
        }

        // ["import", "<module-name>;"]
        let req_name = parse_import(l.to_string());
        requires.push(req_name);
    }

    requires
}

/// import <module-name>; をパースして，モジュール名を切り出す
fn parse_import(l: String) -> String {
    let mut iter = l.split_ascii_whitespace();
    let _ = iter.next(); // import の読み飛ばし
    let import_string = iter.next().unwrap();
    import_string.to_string().trim_end_matches(';').to_string()
}

/// コマンドライン引数に渡されたファイルから内容を読み取ろうとする
/// エラーを発行する可能性もある
fn try_to_get_file_contents(source_name: &str) -> String {
    match fu::read_program_from_file(source_name) {
        Some(contents) => contents,
        None => {
            CE::new(
                BEK::NOTFOUNDSUCHAFILE {
                    file_name: source_name.to_string(),
                },
                Default::default(),
            )
            .output();
            std::process::exit(1);
        }
    }
}

fn setup_startup_routine() -> String {
    match setup::BUILD_OPTION.target {
        opt::Target::X86_64 => format!("{}startup_x64.go", get_lib_path()),
        opt::Target::AARCH64 => format!("{}startup_aarch64.go", get_lib_path()),
    }
}

/// コンパイラのディレクトリに存在するlib/を返す
fn get_lib_path() -> String {
    let lib_path = std::env::var("PEACHILI_LIB_PATH");

    if lib_path.is_err() {
        panic!("`PEACHILI_LIB_PATH` was not found.");
    }

    let lib_path = lib_path.unwrap();
    let ends_with_slash = lib_path.ends_with('/');

    if ends_with_slash {
        lib_path
    } else {
        lib_path + "/"
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::*;

    #[test]
    fn parse_import_test() {
        let actual = parse_import("import A;".to_string());
        assert_eq!("A", actual);
    }

    #[test]
    fn collect_import_modules_from_program_test() {
        // 空行あり
        let s1 = "import A;\nimport B;\nimport C;\n\nstruct A{}\n".to_string();

        let actual = collect_import_modules_from_program(s1);

        assert_eq!(3, actual.len());

        // importなし
        let s2 = "\n\n\n\nstruct A{}\n".to_string();

        let actual = collect_import_modules_from_program(s2);

        assert_eq!(0, actual.len());
    }

    #[test]
    fn search_directory_test() {
        // テスト実行時の相対パスで取っている
        let dir = search_directory("examples".to_string());
        assert!(dir.is_some());
    }

    #[test]
    fn search_peachili_program_test() {
        let file = search_peachili_program("examples/x64/empty_main.go".to_string());
        assert!(file.is_some());
    }

    #[test]
    fn search_module_test() {
        let dir = search_module("examples".to_string());
        assert!(dir.is_some());

        let file = search_module("examples/x64/empty_main.go".to_string());
        assert!(file.is_some());

        let invalid = search_module("invalid".to_string());
        assert!(invalid.is_none());
    }

    #[test]
    #[ignore]
    fn resolve_test() {
        // lib/ のディレクトリ
        let p = resolve_path_from_name("std".to_string());
        assert_eq!(format!("{}std", get_lib_path()), p);

        // lib/ のファイル
        let p = resolve_path_from_name("std/os".to_string());
        assert_eq!(format!("{}std/os.go", get_lib_path()), p);

        // 相対パスのディレクトリ
        let p = resolve_path_from_name("examples".to_string());
        assert_eq!("examples", p);

        // 相対パスのファイル
        let p = resolve_path_from_name("examples/boolean_1.go".to_string());
        assert_eq!("examples/boolean_1.go", p);
    }
}
