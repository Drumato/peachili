use crate::assembler::x64;
use crate::common::arch;

impl x64::Assembler {
    pub fn codegen(&mut self, asm_file: &arch::x64::AssemblyFile) {
        let symbols = asm_file.get_symbols();

        for sym in symbols.iter() {
            let bin_symbol = self.symbol_to_binsymbol(sym);
            self.add_symbol(sym.copy_name(), bin_symbol);
        }
    }

    fn symbol_to_binsymbol(&self, sym: &arch::x64::Symbol) -> arch::x64::BinSymbol {
        let mut bin_symbol = arch::x64::BinSymbol::new_global();

        let instructions = sym.get_insts();

        for inst in instructions.iter() {
            let codes = self.generate_from_inst(inst);
            bin_symbol.add_codes(codes);
        }

        bin_symbol
    }

    fn generate_from_inst(&self, inst: &arch::x64::Instruction) -> Vec<u8> {
        let inst_kind = inst.get_kind();

        match inst_kind {
            arch::x64::InstKind::RET => self.generate_ret(),
            _ => panic!(
                "not implemented generating '{}' in x64_asm",
                inst.to_at_code()
            ),
        }
    }
}
