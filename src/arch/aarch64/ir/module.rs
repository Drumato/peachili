use crate::arch::aarch64::ir;

pub struct Module {
    funcs: Vec<ir::Function>,
}

impl Default for Module {
    fn default() -> Self {
        Self { funcs: Vec::new() }
    }
}
impl Module {
    pub fn push_function(&mut self, f: ir::Function) {
        self.funcs.push(f);
    }

    pub fn to_assembly(&self) -> String {
        let mut module_code = String::new();

        for ir_fn in self.funcs.iter() {
            module_code += &ir_fn.to_assembly();
        }

        module_code
    }
}
