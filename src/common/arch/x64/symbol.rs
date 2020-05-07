extern crate elf_utilities;

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

    pub fn copy_name(&self) -> String {
        self.name.to_string()
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

    pub fn get_insts(&self) -> &Vec<Instruction> {
        &self.insts
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct BinSymbol {
    codes: Vec<u8>,
    is_global: bool,
}

#[allow(dead_code)]
impl BinSymbol {
    fn new(is_g: bool) -> Self {
        Self {
            codes: Vec::new(),
            is_global: is_g,
        }
    }
    pub fn new_global() -> Self {
        Self::new(true)
    }

    pub fn new_local() -> Self {
        Self::new(false)
    }

    pub fn add_codes(&mut self, mut src: Vec<u8>) {
        self.codes.append(&mut src);
    }

    pub fn copy_codes(&self) -> Vec<u8> {
        self.codes.clone()
    }

    pub fn code_length(&self) -> usize {
        self.codes.len()
    }

    pub fn set_code(&mut self, idx: usize, byte: u8) {
        self.codes[idx] = byte;
    }
}
