extern crate elf_utilities;
extern crate indexmap;


type Hash = u64;

#[allow(dead_code)]
pub struct Symbol {
    name: String,
    groups: Vec<x64_asm::Group>,

    // Addend -> 文字列のオフセット
    strings: indexmap::IndexMap<String, Hash>,
}

impl Symbol {
    pub fn new(func_name: String) -> Self {
        Self {
            name: func_name,
            groups: Vec::new(),
            strings: indexmap::IndexMap::new(),
        }
    }

    pub fn copy_name(&self) -> String {
        self.name.to_string()
    }

    pub fn to_at_code(&self) -> String {
        let mut code = format!(".global {}\n", self.name);
        code += &(format!("{}:\n", self.name));

        for group in self.groups.iter() {
            code += &group.to_at_string();
        }

        code += ".section .rodata\n";
        for (contents, hash) in self.strings.iter() {
            code += &(format!(".LS{}:\n", hash));
            code += &(format!("  .string \"{}\"\n", contents));
        }

        code
    }

    pub fn add_inst(&mut self, inst: x64_asm::Instruction) {
        if self.groups.is_empty() {
            let mut g: x64_asm::Group = Default::default();
            g.label = "entry".to_string();

            self.groups.push(g);
        }
        let idx = self.groups.len() - 1;
        self.groups[idx].insts.push(inst);
    }
    pub fn add_string(&mut self, contents: String, hash: Hash) {
        self.strings.insert(contents, hash);
    }

    pub fn get_groups(&self) -> &Vec<x64_asm::Group> {
        &self.groups
    }
    pub fn get_strings(&self) -> &indexmap::IndexMap<String, Hash> {
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
