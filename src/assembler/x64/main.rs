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
            // mov
            arch::x64::InstKind::MOVREGTOREG64(src, dst) => self.generate_movregtoreg64(src, dst),
            arch::x64::InstKind::MOVREGTOMEM64(src, base_reg, offset) => {
                self.generate_movregtomem64(src, base_reg, *offset)
            }
            arch::x64::InstKind::MOVMEMTOREG64(base_reg, offset, src) => {
                self.generate_movmemtoreg64(base_reg, *offset, src)
            }
            arch::x64::InstKind::MOVIMMTOREG64(imm, dst) => self.generate_movimmtoreg64(imm, dst),

            // pop
            arch::x64::InstKind::POPREG64(value) => self.generate_popreg64(value),

            // push
            arch::x64::InstKind::PUSHINT64(immediate) => self.generate_pushint64(immediate),
            arch::x64::InstKind::PUSHREG64(value) => self.generate_pushreg64(value),

            // sub
            arch::x64::InstKind::SUBREGTOREG64(src, dst) => self.generate_subregtoreg64(src, dst),
            arch::x64::InstKind::SUBREGBYUINT64(imm, dst) => self.generate_subregbyuint64(imm, dst),

            // ret
            arch::x64::InstKind::RET => self.generate_ret(),

            // syscall
            arch::x64::InstKind::SYSCALL => self.generate_syscall(),

            // etc.
            _ => panic!(
                "not implemented generating '{}' in x64_asm",
                inst.to_at_code()
            ),
        }
    }
}
