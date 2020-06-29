use std::collections::BTreeMap;
use std::time;

use crate::common::{module, option as opt};
use crate::compiler::general::resource as res;

pub fn allocate_frame_phase(
    build_option: &opt::BuildOption,
    func_map: &mut BTreeMap<res::PStringId, res::PFunction>,
    _module_allocator: &module::ModuleAllocator,
) {
    let start = time::Instant::now();

    for (_func_name_id, func) in func_map.iter_mut() {

        func.alloc_frame();

    }

    let end = time::Instant::now();

    if build_option.verbose {
        eprintln!("allocate frame done!( in {:?})", end - start);
    }
}

impl res::PFunction {
    fn alloc_frame(&mut self) {
        let mut total_offset: usize = 0;
        for (_name, pvar) in self.locals.iter_mut() {
            total_offset += pvar.type_size();
            pvar.set_stack_offset(total_offset);
        }

        self.set_stack_offset(total_offset);
    }
}
