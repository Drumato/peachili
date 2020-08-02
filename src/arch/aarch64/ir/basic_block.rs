use crate::arch::aarch64::ir;

pub struct BasicBlock {
    name: String,
    insts: Vec<ir::Instruction>,
}

impl BasicBlock {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            insts: Vec::new(),
        }
    }

    pub fn push_inst(&mut self, inst: ir::Instruction) {
        self.insts.push(inst);
    }

    pub fn to_assembly(&self) -> String {
        let mut bb_str = format!("\"{}\":\n", self.name);

        for inst in self.insts.iter() {
            bb_str += &format!("    {}\n", inst.to_assembly());
        }

        bb_str
    }
}
