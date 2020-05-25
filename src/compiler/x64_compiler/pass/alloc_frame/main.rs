use std::collections::BTreeMap;

use crate::common::option as opt;
use crate::compiler::general::resource as res;

use std::time;

pub fn allocate_frame_phase(
    _build_option: &opt::BuildOption,
    func_map: &mut BTreeMap<String, res::PFunction>,
) {
    let function_number = func_map.len() as u64;
    let allocate_frame_pb = indicatif::ProgressBar::new(function_number);
    allocate_frame_pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .progress_chars("#>-"),
    );

    let start = time::Instant::now();
    for (func_name, func) in func_map.iter_mut() {
        allocate_frame_pb.set_message(&format!("allocate stack frame in {}", func_name));
        func.alloc_frame();

        allocate_frame_pb.inc(1);
    }
    let end = time::Instant::now();
    allocate_frame_pb
        .finish_with_message(&format!("allocate stack frame done!(in {:?})", end - start));
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
