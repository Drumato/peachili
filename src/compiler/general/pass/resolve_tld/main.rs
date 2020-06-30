use std::collections::BTreeMap;
use std::time;

use crate::common::{module, option as opt};
use crate::compiler::general::resource as res;

pub fn resolve_tld_phase(
    build_option: &opt::BuildOption,
    root: &res::ASTRoot,
    _module_allocator: &module::ModuleAllocator,
) -> BTreeMap<res::PStringId, res::TopLevelDecl> {
    let func_map = root.get_functions();
    let type_map = root.get_typedefs();

    let start = time::Instant::now();

    let mut resolver: res::TLDResolver = Default::default();
    resolver.resolve_typedefs(build_option, type_map);

    for (func_name_id, func) in func_map.iter() {
        resolver.resolve_fn(build_option, *func_name_id, func, type_map);
    }

    let end = time::Instant::now();

    if build_option.verbose {
        eprintln!("resolve tld done!( in {:?})", end - start);
    }

    resolver.give_map()
}

impl res::TLDResolver {
    pub fn resolve_typedefs(
        &mut self,
        _build_option: &opt::BuildOption,
        type_map: &BTreeMap<res::PStringId, res::PType>,
    ) {
        for (type_name_id, def_type) in type_map.iter() {
            let tld_type = if def_type.is_struct() {
                res::TopLevelDecl::new_struct(def_type.copy_members())
            } else {
                res::TopLevelDecl::new_alias(def_type.clone())
            };

            self.insert_entry(*type_name_id, tld_type);
        }
    }

    pub fn resolve_fn(
        &mut self,
        _build_option: &opt::BuildOption,
        func_name_id: res::PStringId,
        func: &res::PFunction,
        type_map: &BTreeMap<res::PStringId, res::PType>,
    ) {
        let return_type = func.get_return_type().clone();
        let args = self.collect_arg_types(func, type_map);

        let tld_fn = res::TopLevelDecl::new_fn(return_type, args);
        self.insert_entry(func_name_id, tld_fn);
    }

    fn collect_arg_types(
        &mut self,
        func: &res::PFunction,
        type_map: &BTreeMap<res::PStringId, res::PType>,
    ) -> Vec<(res::PStringId, res::PType)> {
        let mut args: Vec<(res::PStringId, res::PType)> = Vec::new();
        let arg_types = func.collect_arg_types(type_map);

        for (arg_idx, arg_name_id) in func.get_args().iter().enumerate() {
            args.push((*arg_name_id, arg_types[arg_idx].clone()));
        }

        args
    }
}
