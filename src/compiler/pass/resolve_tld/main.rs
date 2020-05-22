use std::collections::BTreeMap;
use std::time;

use crate::common::option as opt;
use crate::compiler::resource as res;

pub fn resolve_tld_phase(
    build_option: &opt::BuildOption,
    root: &res::ASTRoot,
) -> BTreeMap<String, res::TopLevelDecl> {
    let func_map = root.get_functions();
    let type_map = root.get_typedefs();

    let function_number = func_map.len() as u64;
    let resolve_tld_pb = indicatif::ProgressBar::new(function_number);
    resolve_tld_pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("#>-"),
    );

    let start = time::Instant::now();

    let mut resolver: res::TLDResolver = Default::default();
    resolver.resolve_typedefs(build_option, type_map);

    for (func_name, func) in func_map.iter() {
        resolve_tld_pb.set_message(&format!("resolve tld in {}", func_name));

        resolver.resolve_fn(build_option, func_name, func, type_map);

        resolve_tld_pb.inc(1);
    }

    let end = time::Instant::now();
    resolve_tld_pb.finish_with_message(&format!("resolve tld done!(in {:?})", end - start));

    resolver.give_map()
}

impl res::TLDResolver {
    pub fn resolve_typedefs(
        &mut self,
        _build_option: &opt::BuildOption,
        type_map: &BTreeMap<String, res::PType>,
    ) {
        for (type_name, def_type) in type_map.iter() {
            let tld_type = res::TopLevelDecl::new_alias(def_type.clone());

            self.insert_entry(type_name.to_string(), tld_type);
        }
    }

    pub fn resolve_fn(
        &mut self,
        _build_option: &opt::BuildOption,
        func_name: &str,
        func: &res::PFunction,
        type_map: &BTreeMap<String, res::PType>,
    ) {
        let return_type = func.get_return_type().clone();
        let args = self.collect_arg_types(func, type_map);

        let tld_fn = res::TopLevelDecl::new_fn(return_type, args);
        self.insert_entry(func_name.to_string(), tld_fn);
    }

    fn collect_arg_types(
        &mut self,
        func: &res::PFunction,
        type_map: &BTreeMap<String, res::PType>,
    ) -> Vec<(String, res::PType)> {
        let mut args: Vec<(String, res::PType)> = Vec::new();
        let arg_types = func.collect_arg_types(type_map);

        for (arg_idx, arg_name) in func.get_args().iter().enumerate() {
            args.push((arg_name.to_string(), arg_types[arg_idx].clone()));
        }

        args
    }
}
