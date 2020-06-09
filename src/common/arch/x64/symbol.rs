extern crate elf_utilities;

use std::collections::BTreeMap;

use crate::common::arch::x64::*;

type Hash = u64;

#[allow(dead_code)]
pub struct Symbol {
    name: String,
    insts: Vec<Instruction>,

    // Addend -> 文字列のオフセット
    strings: BTreeMap<String, Hash>,
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

        code += ".section .rodata\n";
        for (contents, hash) in self.strings.iter() {
            code += &(format!(".LS{}:\n", hash));
            code += &(format!("  .string \"{}\"\n", contents));
        }

        code
    }
    pub fn add_inst(&mut self, inst: Instruction) {
        self.insts.push(inst);
    }
    pub fn add_string(&mut self, contents: String, hash: Hash) {
        self.strings.insert(contents, hash);
    }

    pub fn get_insts(&self) -> &Vec<Instruction> {
        &self.insts
    }
    pub fn get_strings(&self) -> &BTreeMap<String, Hash> {
        &self.strings
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct BinSymbol {
    name: Option<String>,
    codes: Vec<u8>,
    is_global: bool,
    strings: Vec<String>,
}

#[allow(dead_code)]
impl BinSymbol {
    fn new(name: Option<String>, is_g: bool) -> Self {
        Self {
            name,
            codes: Vec::new(),
            is_global: is_g,
            strings: Vec::new(),
        }
    }
    pub fn new_global(name: Option<String>) -> Self {
        Self::new(name, true)
    }

    pub fn new_local(name: Option<String>) -> Self {
        Self::new(name, false)
    }

    pub fn add_codes(&mut self, mut src: Vec<u8>) {
        self.codes.append(&mut src);
    }

    pub fn add_string_literal(&mut self, literal: String) {
        self.strings.push(literal);
    }

    pub fn copy_strings(&self) -> Vec<String> {
        self.strings.clone()
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
