use crate::{module as m, option as opt};
use typed_arena::Arena;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

/// Peachiliプログラムにつける拡張子
const PEACHILI_FILE_EXTENSION: &'static str = ".go";

pub fn resolve_main<'a>(
    target: opt::Target,
    arena: &'a Arena<m::ModuleInfo<'a>>,
    source_name: String,
) -> m::Module<'a> {
    let file_contents = try_to_get_file_contents(&source_name);
    // mainが参照するモジュールに対しそれぞれprocess_ext_moduleする
    let main_requires = collect_import_modules_from_program(&file_contents);

    let main_module = arena.alloc(m::ModuleInfo::new_primitive(
        PathBuf::from(source_name),
        "main".to_string(),
        file_contents.clone(),
    ));

    // スタートアップ･ライブラリの追加
    let startup_module_path = setup_startup_routine(target);
    let startup_module = analyze_external_module(arena, startup_module_path, "startup".to_string());
    if let m::ModuleKind::Primitive { refs, contents: _ } = &main_module.kind {
        refs.as_ref().borrow_mut().push(startup_module);
    }

    add_dependencies_to(arena, main_module, main_requires);

    main_module
}

/// (間接的にではあるが)再帰的に呼び出される
fn analyze_external_module<'a>(
    arena: &'a Arena<m::ModuleInfo<'a>>,
    external_file_path: String,
    external_module_name: String,
) -> m::Module<'a> {
    let external_file_path = PathBuf::from(external_file_path);

    // TODO: エラー出したほうがいいかも
    let parent_module_is_dir = external_file_path.is_dir();

    let parent_module = if parent_module_is_dir {
        // parent_modのsubsにモジュールをぶら下げる
        create_directory_module(arena, external_file_path, external_module_name)
    } else {
        // 普通のファイルと同じように処理する
        create_primitive_module(arena, external_file_path, external_module_name)
    };

    parent_module
}

/// ディレクトリ内の各ファイルに対して，resolveを実行する
fn create_directory_module<'a>(
    arena: &'a Arena<m::ModuleInfo<'a>>,
    file_path: PathBuf,
    module_name: String,
) -> m::Module<'a> {
    let mut children = Vec::new();
    for entry in fs::read_dir(&file_path).unwrap() {
        let file_in_dir = entry.unwrap();
        let child_module_name = file_in_dir.path().to_str().unwrap().to_string();
        let resolved_path = resolve_path_from_name(child_module_name.to_string());

        let child_module = analyze_external_module(arena, resolved_path, child_module_name);
        children.push(child_module);
    }

    let mut parent_module = m::ModuleInfo::new_directory(file_path, module_name);
    if let m::ModuleKind::Directory {
        children: ref mut c,
    } = parent_module.kind
    {
        *c = Rc::new(RefCell::new(children));
    }

    arena.alloc(parent_module)
}

fn create_primitive_module<'a>(
    arena: &'a Arena<m::ModuleInfo<'a>>,
    external_file_path: PathBuf,
    external_module_name: String,
) -> m::Module<'a> {
    let file_contents = try_to_get_file_contents(external_file_path.to_str().unwrap());
    let requires = collect_import_modules_from_program(&file_contents);
    let m = arena.alloc(m::ModuleInfo::new_primitive(
        external_file_path.clone(),
        external_module_name,
        file_contents,
    ));

    add_dependencies_to(arena, m, requires);

    m
}

/// 依存ノードを追加する
fn add_dependencies_to<'a>(
    arena: &'a Arena<m::ModuleInfo<'a>>,
    src_module: m::Module<'a>,
    dependencies: Vec<String>,
) {
    for req in dependencies {
        let req_path = resolve_path_from_name(req.to_string());
        let referenced_module = analyze_external_module(arena, req_path, req);

        if let m::ModuleKind::Primitive { refs, contents: _ } = &src_module.kind {
            refs.as_ref().borrow_mut().push(referenced_module);
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
    let resolved_dir = find_dir_package(module_name.to_string());

    if let Some(dir_path) = resolved_dir {
        return Some(dir_path);
    }

    // ディレクトリがなかったので，拡張子をつけて再度チェック
    let resolved_file =
        search_peachili_program(format!("{}{}", module_name, PEACHILI_FILE_EXTENSION));
    if let Some(file_path) = resolved_file {
        return Some(file_path);
    }

    None
}

/// 引数に渡したディレクトリが存在するかチェック
fn search_peachili_program(file_name: String) -> Option<String> {
    if !file_name.ends_with(PEACHILI_FILE_EXTENSION) {
        return None;
    }

    let metadata = fs::metadata(file_name.to_string());

    // そもそもファイルが存在しなかった
    if metadata.is_err() {
        return None;
    }

    // 拡ソースファイルを発見した
    Some(file_name)
}

/// 引数に渡したPeachiliファイルが存在するかチェック
fn find_dir_package(dir_name: String) -> Option<String> {
    if dir_name.ends_with(PEACHILI_FILE_EXTENSION) {
        return None;
    }

    let metadata = fs::metadata(dir_name.to_string());

    // そもそもファイルが存在しなかった
    if metadata.is_err() {
        return None;
    }

    // 何も拡張子をつけないでファイルを見つけられた -> ディレクトリを発見した
    Some(dir_name)
}

/// ファイル先頭にある任意数の `import <module-name>;` を解読して返す
fn collect_import_modules_from_program(file_contents: &str) -> Vec<String> {
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
fn try_to_get_file_contents(file_name: &str) -> String {
    match std::fs::read_to_string(file_name) {
        Ok(contents) => contents,
        Err(e) => {
            panic!("{}: {}", file_name, e);
        }
    }
}

fn setup_startup_routine(target: opt::Target) -> String {
    match target {
        opt::Target::X86_64 => format!("{}startup_x64.go", get_lib_path()),
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
    use m::ModuleKind;

    use super::*;

    #[test]
    fn parse_import_test() {
        let actual = parse_import("import A;".to_string());
        assert_eq!("A", actual);
    }

    #[test]
    fn collect_import_modules_from_program_test() {
        // 空行あり
        let s1 = "import A;\nimport B;\nimport C;\n\nstruct A{}\n";

        let actual = collect_import_modules_from_program(s1);

        assert_eq!(3, actual.len());

        // importなし
        let s2 = "\n\n\n\nstruct A{}\n";

        let actual = collect_import_modules_from_program(s2);

        assert_eq!(0, actual.len());
    }

    #[test]
    fn search_directory_test() {
        // テスト実行時の相対パスで取っている
        let dir = find_dir_package("examples".to_string());
        assert!(dir.is_some());
    }

    #[test]
    fn search_peachili_program_test() {
        let file = search_peachili_program("examples/x64/intlit.go".to_string());
        assert!(file.is_some());
    }

    #[test]
    fn search_module_test() {
        let dir = search_module("examples".to_string());
        assert!(dir.is_some());

        let file = search_module("examples/x64/intlit".to_string());
        assert!(file.is_some());

        let invalid = search_module("invalid".to_string());
        assert!(invalid.is_none());
    }

    #[test]
    fn analyze_external_module_test() {
        let arena = Default::default();
        std::env::set_var("PEACHILI_LIB_PATH", "./lib");
        let a_module = analyze_external_module(
            &arena,
            "src/bundler/test_data/a.go".to_string(),
            "src/bundler/test_data/a".to_string(),
        );
        assert_eq!("src::bundler::test_data::a", a_module.name);
        assert_eq!(
            "src/bundler/test_data/a.go",
            a_module.file_path.to_str().unwrap()
        );
        if let ModuleKind::Primitive { refs, contents: _ } = &a_module.kind {
            assert_eq!(1, refs.as_ref().borrow().len());
        }
    }
}
