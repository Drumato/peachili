use std::collections::BTreeMap;
use std::time;

use crate::common::{module, option as opt};
use crate::compiler::general::resource as res;

pub fn resolve_unknown_type_phase(
    build_option: &opt::BuildOption,
    func_map: &mut BTreeMap<res::PStringId, res::PFunction>,
    tld_map: &BTreeMap<res::PStringId, res::TopLevelDecl>,
    _module_allocator: &module::ModuleAllocator,
) {
    let start = time::Instant::now();

    for (_func_name_id, func) in func_map.iter_mut() {

        func.resolve_type(tld_map);

    }

    let end = time::Instant::now();

    if build_option.verbose {
        eprintln!("type resolve done!( in {:?})", end - start);
    }
}

impl res::PFunction {
    fn resolve_type(&mut self, tld_map: &BTreeMap<res::PStringId, res::TopLevelDecl>) {
        for (_name, pvar) in self.locals.iter_mut() {
            let current_type = pvar.get_type();

            if let res::PTypeKind::UNRESOLVED(type_name) = &current_type.kind {
                let type_last = res::IdentName::last_name(type_name);
                if tld_map.get(&type_last).is_none() {
                    panic!("should emit a compile-error");
                }

                let base_type = tld_map.get(&type_last).unwrap();

                let resolved_type = base_type.to_ptype();
                pvar.set_type(resolved_type.clone());
            }
        }
    }
}
