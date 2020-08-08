use crate::arch::x64::ir;

pub struct Function {
    name: String,
    blocks: Vec<ir::BasicBlock>,
}

impl Function {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            blocks: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn push_block(&mut self, name: &str) {
        self.blocks
            .push(ir::BasicBlock::new(&format!("{}_{}", self.name, name)));
    }

    pub fn add_inst_to_last_bb(&mut self, inst: ir::Instruction) {
        let last_bb = self.blocks.len() - 1;

        self.blocks[last_bb].push_inst(inst);
    }

    pub fn to_atandt(&self) -> String {
        let mut func_code = format!(".global \"{}\"\n", self.name);
        func_code += &format!("\"{}\":\n", self.name);

        for bb in self.blocks.iter() {
            func_code += &format!("  {}\n", bb.to_atandt());
        }

        func_code
    }
}
