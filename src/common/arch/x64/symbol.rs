use std::collections::BTreeMap;

use crate::common::arch::x64::*;

#[allow(dead_code)]
pub struct Symbol {
    name: String,
    insts: Vec<Instruction>,
    strings: BTreeMap<String, u64>,
}

impl Symbol {
    pub fn new(func_name: String) -> Self {
        Self {
            name: func_name,
            insts: Vec::new(),
            strings: BTreeMap::new(),
        }
    }

    pub fn to_at_code(&self) -> String {
        let mut code = format!(".global {}\n", self.name);
        code += &(format!("{}:\n", self.name));

        for ins in self.insts.iter() {
            code += &(format!("  {}\n", ins.to_at_code()));
        }

        for (contents, hash) in self.strings.iter() {
            code += &(format!(".LS{}:\n", hash));
            code += &(format!("  .string \"{}\"\n", contents));
        }

        code
    }
    pub fn add_inst(&mut self, inst: Instruction) {
        self.insts.push(inst);
    }
    pub fn add_string(&mut self, contents: String, hash: u64) {
        self.strings.insert(contents, hash);
    }
}
