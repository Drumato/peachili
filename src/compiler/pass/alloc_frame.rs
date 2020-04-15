use crate::compiler::resource as res;

impl res::PFunction {
    pub fn alloc_frame(&mut self) {
        let mut total_offset: usize = 0;
        for (_name, pvar) in self.locals.iter_mut() {
            total_offset += pvar.type_size();
            pvar.set_stack_offset(total_offset);
        }

        self.set_stack_offset(total_offset);
    }
}
