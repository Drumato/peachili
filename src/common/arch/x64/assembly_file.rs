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

    pub fn to_intelcode(&self) -> String {
        let mut code = ".intel_syntax noprefix\n".to_string();
        for sym in self.symbols.iter() {
            code += &sym.to_intelcode();
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
    pub fn symbols_number(&self) -> usize {
        self.symbols.len()
    }
}
