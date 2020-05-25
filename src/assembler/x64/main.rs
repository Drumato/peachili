extern crate elf_utilities;

use std::collections::HashMap;

use crate::assembler::x64;
use crate::common::{arch, option};

type CodeIndex = usize;
type Offset = usize;

pub fn x64_assemble(
    build_option: &option::BuildOption,
    asm_file: arch::x64::AssemblyFile,
) -> x64::ELFBuilder {
    if build_option.verbose {
        eprintln!("start assembling...");
    }

    let mut generator: x64::Assembler = Default::default();
    generator.codegen(&asm_file);

    // 再配置情報の更新
    // シンボルテーブルを検索して，再配置テーブルに存在すれば情報更新
    generator.setup_relocation_informations(&asm_file);

    // オブジェクトファイル生成
    let object_file = new_object_file();
    let mut object_file_builder = x64::ELFBuilder::new(object_file);

    // (NULL) セクション
    object_file_builder.add_section(elf_utilities::section::Section64::new_null_section());
    // .text セクション
    object_file_builder.add_text_section(&generator);
    // .symtab セクション
    object_file_builder.add_symbol_table_section(&generator);
    // .strtab セクション
    object_file_builder.add_symtab_string_section(&generator);
    // .rela.text セクション
    object_file_builder.add_relatext_section(&generator);
    // .rodata セクション
    object_file_builder.add_rodata_section(&generator);
    // .shstrtab セクション
    object_file_builder.add_shstrtab_string_section();

    // ヘッダの調整
    object_file_builder.condition_elf_header();

    object_file_builder
}

impl x64::Assembler {
    pub fn codegen(&mut self, asm_file: &arch::x64::AssemblyFile) {
        let symbols = asm_file.get_symbols();

        for sym in symbols.iter() {
            let (mut bin_symbol, jump_map) = self.symbol_to_binsymbol(sym);

            // ジャンプ系命令の解決
            self.resolve_jump_instructions(&mut bin_symbol, &jump_map);
            self.add_symbol(sym.copy_name(), bin_symbol);
        }
    }

    fn symbol_to_binsymbol(
        &mut self,
        sym: &arch::x64::Symbol,
    ) -> (arch::x64::BinSymbol, HashMap<String, (CodeIndex, Offset)>) {
        let mut bin_symbol = arch::x64::BinSymbol::new_global();
        let instructions = sym.get_insts();

        // シンボルごとに初期化
        self.set_byte_length(0);
        let mut jump_map: HashMap<String, (CodeIndex, Offset)> = HashMap::new();

        let sym_name = sym.copy_name();
        for inst in instructions.iter() {
            let codes = self.generate_from_inst(inst, &sym_name, &mut jump_map);

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

        // 文字列群のコピー
        let strings = sym.get_strings();

        for (contents, _hash) in strings.iter() {
            bin_symbol.add_string_literal(contents.to_string());
        }

        (bin_symbol, jump_map)
    }

    fn generate_from_inst(
        &mut self,
        inst: &arch::x64::Instruction,
        sym_name: &str,
        jump_map: &mut HashMap<String, (CodeIndex, Offset)>,
    ) -> Vec<u8> {
        let inst_kind = inst.get_kind();

        match inst_kind {
            // ラベル
            arch::x64::InstKind::LABEL(name) => {
                let length = self.get_all_byte_length() as usize;

                // jump系命令がラベルの前に存在した場合
                if let Some(tup) = jump_map.get_mut(name) {
                    // ラベルまでのバイト数 - ジャンプの位置 - 1 => 相対オフセット
                    tup.1 = length - tup.1 - 1;
                    return Vec::new();
                }

                // ラベルがjump系命令の前に存在した場合
                jump_map.insert(name.to_string(), (0, length));

                Vec::new()
            }

            // lea
            arch::x64::InstKind::LEASTRINGTOREGWITHRIP(label, dst) => {
                // 再配置用にシンボルを定義
                let mut rela: elf_utilities::relocation::Rela64 = Default::default();
                rela.set_addend(-4);
                let offset_before_lea = self.get_all_byte_length();

                // 3 -> immediateまでスキップ generate_lea64を見るとわかる
                rela.set_offset(offset_before_lea + 3);

                // 2 -> .rodataのsymtabインデックス決め打ち
                rela.set_info((2 << 32) + elf_utilities::relocation::R_X86_64_PC32);

                self.add_relocation_symbol(sym_name.to_string(), label.to_string(), rela);

                self.generate_learegtoregwithrip64(&arch::x64::Immediate::new_int64(0), dst)
            }

            // jump
            arch::x64::InstKind::JUMPEQUAL(name) => {
                let length = (self.get_all_byte_length() + 2) as usize;

                if let Some(tup) = jump_map.get_mut(name) {
                    // ラベルがjump系命令の前に存在した場合
                    tup.0 = length;
                    tup.1 = !(length + 4 - tup.1) + 1;
                } else {
                    // jump系命令がラベルの前に存在した場合
                    jump_map.insert(name.to_string(), (length, length + 3));
                }

                // opcode
                let opcode1 = 0x0f;
                let opcode2 = 0x84;

                let mut codes = vec![opcode1, opcode2];
                // immediate-value
                for b in (0x00 as u32).to_le_bytes().to_vec().iter() {
                    codes.push(*b);
                }

                codes
            }
            arch::x64::InstKind::JUMP(name) => {
                let length = (self.get_all_byte_length() + 1) as usize;

                if let Some(tup) = jump_map.get_mut(name) {
                    // ラベルがjump系命令の前に存在した場合
                    tup.0 = length;
                    tup.1 = !(length + 4 - tup.1) + 1;
                } else {
                    // jump系命令がラベルの前に存在した場合
                    jump_map.insert(name.to_string(), (length, length + 3));
                }

                // opcode
                let opcode = 0xe9;

                let mut codes = vec![opcode];
                // immediate-value
                for b in (0x00 as u32).to_le_bytes().to_vec().iter() {
                    codes.push(*b);
                }

                codes
            }

            // add
            arch::x64::InstKind::ADDREGTOREG64(src, dst) => self.generate_addregtoreg64(src, dst),

            // cmp
            arch::x64::InstKind::CMPREGANDINT64(value, dst) => {
                self.generate_cmpregandint64(value, dst)
            }
            arch::x64::InstKind::CMPREGANDREG64(src, dst) => self.generate_cmpregandreg64(src, dst),

            // imul
            arch::x64::InstKind::IMULREGTOREG64(src, dst) => self.generate_imulregtoreg64(src, dst),

            // mov
            arch::x64::InstKind::MOVREGTOREG64(src, dst) => self.generate_movregtoreg64(src, dst),
            arch::x64::InstKind::MOVREGTOMEM64(src, base_reg, offset) => {
                self.generate_movregtomem64(src, base_reg, *offset)
            }
            arch::x64::InstKind::MOVMEMTOREG64(base_reg, offset, src) => {
                self.generate_movmemtoreg64(base_reg, *offset, src)
            }
            arch::x64::InstKind::MOVIMMTOREG64(imm, dst) => self.generate_movimmtoreg64(imm, dst),

            // inc
            arch::x64::InstKind::INCREG64(value) => self.generate_increg64(value),

            // neg
            arch::x64::InstKind::NEGREG64(value) => self.generate_negreg64(value),

            // pop
            arch::x64::InstKind::POPREG64(value) => self.generate_popreg64(value),

            // push
            arch::x64::InstKind::PUSHINT64(immediate) => self.generate_pushint64(immediate),
            arch::x64::InstKind::PUSHUINT64(immediate) => self.generate_pushint64(immediate),
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

    fn resolve_jump_instructions(
        &self,
        bin_sym: &mut arch::x64::BinSymbol,
        jump_map: &HashMap<String, (CodeIndex, Offset)>,
    ) {
        for (_name, (dst, offset)) in jump_map.iter() {
            for (idx, byte) in (*offset as u32).to_le_bytes().iter().enumerate() {
                bin_sym.set_code(idx + dst, *byte);
            }
        }
    }
}

fn new_object_file() -> elf_utilities::file::ELF64 {
    let ehdr = initialize_elf64_header();

    elf_utilities::file::ELF64::new(ehdr)
}

fn initialize_elf64_header() -> elf_utilities::header::Ehdr64 {
    let mut ehdr: elf_utilities::header::Ehdr64 = Default::default();

    // アーキテクチャ -> X86_64
    ehdr.set_machine(elf_utilities::header::ELFMACHINE::EMX8664);

    // クラス -> 64bit
    ehdr.set_class(elf_utilities::header::ELFCLASS::CLASS64);

    // タイプ -> RELOCATION
    ehdr.set_elf_type(elf_utilities::header::ELFTYPE::REL);

    // データ -> Little Endian
    ehdr.set_data(elf_utilities::header::ELFDATA::DATA2LSB);

    // バージョン -> EV_CURRENT
    ehdr.set_version(elf_utilities::header::ELFVERSION::VERSIONCURRENT);

    ehdr
}
