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

    fn symbol_to_binsymbol(&mut self, sym: &arch::x64::Symbol) -> arch::x64::BinSymbol {
        let mut bin_symbol = arch::x64::BinSymbol::new_global();
        let instructions = sym.get_insts();

        // シンボルごとに初期化
        self.set_byte_length(0);

        let sym_name = sym.copy_name();
        for inst in instructions.iter() {
            let codes = self.generate_from_inst(inst, &sym_name);

            // call命令のオフセットを計算するため，
            // コード長は命令ごとに更新しておく．
            self.add_byte_length(codes.len() as u64);

            bin_symbol.add_codes(codes);
        }

        // アラインメント調整
        let mut extra_bytes: Vec<u8> = Vec::new();
        let rest_bytes = bin_symbol.code_length() % 4;
        for _ in 0..(4 - rest_bytes) {
            extra_bytes.push(0x00);
        }
        bin_symbol.add_codes(extra_bytes);

        bin_symbol
    }

    fn generate_from_inst(&mut self, inst: &arch::x64::Instruction, sym_name: &str) -> Vec<u8> {
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
            arch::x64::InstKind::COMMENT(_contents) => Vec::new(),
            arch::x64::InstKind::CALL(callee_name) => {
                // 再配置用にシンボルを定義
                let mut rela: elf_utilities::relocation::Rela64 = Default::default();
                rela.set_addend(-4);

                // 1 -> オペコード分スキップ generate_callrm64を見るとわかる
                let offset_before_call = self.get_all_byte_length();
                rela.set_offset(offset_before_call + 1);

                self.add_relocation_symbol(sym_name.to_string(), callee_name.to_string(), rela);

                self.generate_callrm64()
            }
            _ => panic!(
                "not implemented generating '{}' in x64_asm",
                inst.to_at_code()
            ),
        }
    }
}
