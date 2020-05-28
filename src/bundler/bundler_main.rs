use std::{
    collections::BTreeMap,
    fs,
    sync::{Arc, Mutex},
};

use crate::bundler::{bundle_parser as bp, resolver};
use crate::common::{module, operate, option};
use crate::compiler::general::{pass, resource as res};

pub fn bundle_main(
    build_option: &option::BuildOption,
    main_fp: String,
    module_allocator: &mut module::ModuleAllocator,
    const_pool: Arc<Mutex<res::ConstAllocator>>,
) -> module::ModuleId {
    // buffer_cacheを用いて，すでにアロケートしたPStringIdを使い回すようにする．
    // ライフタイムはbundle_main内なので，そこまでメモリを圧迫することはないと思う
    let mut module_resolver = resolver::Resolver::new(module_allocator);
    let buffer_cache: Arc<Mutex<BTreeMap<String, res::PStringId>>> =
        Arc::new(Mutex::new(BTreeMap::new()));

    // mainモジュールのアロケート．
    // 必ずソースファイルになっているはずなので，そのまま require を処理する
    let main_id = module_resolver.alloc_main_module(main_fp);
    module_resolver.tokenize_source_module(
        build_option,
        main_id,
        const_pool.clone(),
        buffer_cache.clone(),
    );

    module_resolver.proc_requires(build_option, main_id, const_pool, buffer_cache);

    main_id
}

impl<'a> resolver::Resolver<'a> {
    fn tokenize_source_module(
        &mut self,
        build_option: &option::BuildOption,
        mod_id: module::ModuleId,
        const_pool: Arc<Mutex<res::ConstAllocator>>,
        buffer_cache: Arc<Mutex<BTreeMap<String, res::PStringId>>>,
    ) {
        let mut_module_wrap = self.get_module_as_mut(mod_id);

        if mut_module_wrap.is_none() {
            panic!("not found such a module id -> {:?}", mod_id);
        }

        let mut_module = mut_module_wrap.unwrap();

        // モジュール名をvalidなパスに変換し，格納
        let file_path = construct_file_path(&mut_module.file_path);
        mut_module.set_file_path(file_path);

        let contents = operate::read_program_from_file(&mut_module.file_path);
        let (tokens, const_arena) =
            pass::tokenize_phase(build_option, mut_module, contents, const_pool, buffer_cache);

        mut_module.set_tokens(tokens);
        mut_module.set_const_arena(const_arena);
    }

    fn proc_requires(
        &mut self,
        build_option: &option::BuildOption,
        source_id: module::ModuleId,
        const_pool: Arc<Mutex<res::ConstAllocator>>,
        buffer_cache: Arc<Mutex<BTreeMap<String, res::PStringId>>>,
    ) {
        let requires_names: Vec<res::PStringId>;
        {
            self.tokenize_source_module(
                build_option,
                source_id,
                const_pool.clone(),
                buffer_cache.clone(),
            );

            let source_module = self.get_module_as_mut(source_id).unwrap();
            let mut bundle_parser = bp::BundleParser::new(source_module.get_tokens_as_mut());

            // `require` 無し -> 末端モジュールなのでパース終了
            if !bundle_parser.require_found() {
                return;
            }

            // require の中身をすべて収集する
            requires_names = bundle_parser.parse_each_modules();
        }

        // 各要求モジュールを再帰的に探索
        for required_name_id in requires_names {
            let required_name = {
                let source_module = self.get_module_ref(source_id).unwrap();
                source_module
                    .get_const_pool_ref()
                    .get(required_name_id)
                    .unwrap()
                    .copy_value()
            };

            let required_path = construct_file_path(&required_name);
            let required_module_id = self.proc_external_module(
                build_option,
                required_path,
                const_pool.clone(),
                buffer_cache.clone(),
            );

            let source_module = self.get_module_as_mut(source_id).unwrap();
            let mut required_modules = source_module.get_locked_requires();

            required_modules.push(required_module_id);
        }

        self.set_visited_to_given_id(source_id, true);
    }

    fn proc_submodules(
        &mut self,
        build_option: &option::BuildOption,
        parent_id: module::ModuleId,
        const_pool: Arc<Mutex<res::ConstAllocator>>,
        buffer_cache: Arc<Mutex<BTreeMap<String, res::PStringId>>>,
    ) {
        let parent_module_path: String;

        {
            let parent_module = self.get_module_ref(parent_id).unwrap();
            parent_module_path = parent_module.file_path.clone();
        }

        // ディレクトリ内の各ファイルを再帰的に探索
        for entry in fs::read_dir(&parent_module_path).unwrap() {
            let child_file = entry.unwrap();
            let child_file_path_part = child_file.path().to_str().unwrap().to_string();
            let child_file_path = construct_file_path(&child_file_path_part);

            let sub_module_id = self.proc_external_module(
                build_option,
                child_file_path,
                const_pool.clone(),
                buffer_cache.clone(),
            );

            let parent_module = self.get_module_as_mut(parent_id).unwrap();
            let mut sub_modules = parent_module.get_locked_submodules();
            sub_modules.push(sub_module_id);
        }
    }

    fn proc_external_module(
        &mut self,
        build_option: &option::BuildOption,
        ext_path: String,
        const_pool: Arc<Mutex<res::ConstAllocator>>,
        buffer_cache: Arc<Mutex<BTreeMap<String, res::PStringId>>>,
    ) -> module::ModuleId {
        let ext_mod_id = self.alloc_external_module(ext_path.to_string());

        if fs::metadata(&ext_path).unwrap().is_dir() {
            self.proc_submodules(build_option, ext_mod_id, const_pool, buffer_cache);
        } else {
            self.proc_requires(build_option, ext_mod_id, const_pool, buffer_cache);
        }

        ext_mod_id
    }
}

fn construct_file_path(module_path: &str) -> String {
    // 相対パス中からチェック
    let metadata = fs::metadata(module_path);

    if metadata.is_ok() {
        // ファイルが見つかったので，普通に返す
        return module_path.to_string();
    } else {
        // .go をつけて再度検索
        let extended = format!("{}.go", module_path);

        let metadata = fs::metadata(&extended);
        if metadata.is_ok() {
            return extended;
        }
    }

    // 環境変数からチェック
    let combined_path = combined_libpath_and_file(&module_path);

    let metadata = fs::metadata(&combined_path);

    if metadata.is_ok() {
        return combined_path;
    }

    // .go をつけて再度チェック
    let extended = format!("{}.go", combined_path);
    let metadata = fs::metadata(&extended);

    if metadata.is_err() {
        // エラー
        panic!("not found such a module -> {}", combined_path);
    }

    extended
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
