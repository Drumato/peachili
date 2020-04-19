use crate::common::arch::x64::*;

#[allow(dead_code)]
pub struct AssemblyFile {
    pub file_path: String,
    // strings : BTreeMap<String, String>, Label -> Contents
    symbols: Vec<Symbol>,
    // options: AssemblyOption,  intel syntaxにするか，32/64bit用アセンブリか，
}

impl AssemblyFile {
    pub fn new(fp: String) -> Self {
        Self {
            file_path: fp,
            symbols: Vec::new(),
        }
    }

    pub fn to_at_code(&self) -> String {
        let mut code = String::new();
        for sym in self.symbols.iter() {
            code += &sym.to_at_code();
            code += "\n";
        }
        code
    }

    pub fn add_symbol(&mut self, sym: Symbol) {
        self.symbols.push(sym);
    }
    pub fn add_inst_to_sym(&mut self, idx: usize, inst: Instruction) {
        self.symbols[idx].add_inst(inst);
    }
    pub fn add_string_to_sym(&mut self, idx: usize, contents: String, hash: u64) {
        self.symbols[idx].add_string(contents, hash);
    }
    pub fn symbols_number(&self) -> usize {
        self.symbols.len()
    }
}
