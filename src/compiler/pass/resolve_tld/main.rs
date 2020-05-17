use crate::common::option as opt;
use crate::compiler::resource as res;

impl res::TLDResolver {
    pub fn resolve_fn(
        &mut self,
        _build_option: &opt::BuildOption,
        func_name: &str,
        func: &res::PFunction,
    ) {
        let return_type = res::PType::get_global_type_from(func.get_return_type());
        let mut args: Vec<(String, res::PType)> = Vec::new();
        let arg_types = func.collect_arg_types();

        for (arg_idx, arg_name) in func.get_args().iter().enumerate() {
            args.push((arg_name.to_string(), arg_types[arg_idx].clone()));
        }

        let tld_fn = res::TopLevelDecl::new_fn(return_type, args);
        self.insert_entry(func_name.to_string(), tld_fn);
    }
}
