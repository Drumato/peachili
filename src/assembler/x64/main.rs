extern crate elf_utilities;

use std::collections::{HashMap};

use crate::assembler::x64;
use crate::common::{arch, option};

use x64_asm::*;

type CodeIndex = isize;
type Offset = isize;

pub fn x64_assemble(
    build_option: &option::BuildOption,
    asm_file: arch::x64::AssemblyFile,
) -> arch::x64::ELFBuilder {
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
    let mut object_file_builder = arch::x64::ELFBuilder::new(object_file);

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
    // .nodata セクション
    object_file_builder.add_nodata_section();
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
        let mut bin_symbol = arch::x64::BinSymbol::new_global(Some(sym.copy_name()));
        // シンボルごとに初期化
        self.set_byte_length(0);
        let mut jump_map: HashMap<String, (CodeIndex, Offset)> = HashMap::new();

        // 文字列群のコピー
        let strings = sym.get_strings();
        for (contents, _hash) in strings.iter() {
            bin_symbol.add_string_literal(contents.to_string());
        }

        let sym_name = sym.copy_name();

        let groups = sym.get_groups();
        for group in groups.iter() {
            bin_symbol.add_codes(self.generate_from_group(group, &sym_name, &mut jump_map));

            eprintln!("generating {} finisied -> 0x{:x}", group.label, self.get_all_byte_length());
        }

        // アラインメント調整
        let mut extra_bytes: Vec<u8> = Vec::new();
        let rest_bytes = bin_symbol.code_length() % 4;
        for _ in 0..(4 - rest_bytes) {
            extra_bytes.push(0x00);
        }
        bin_symbol.add_codes(extra_bytes);

        (bin_symbol, jump_map)
    }

    fn generate_from_group(
        &mut self,
        group: &x64_asm::Group,
        sym_name: &str,
        jump_map: &mut HashMap<String, (CodeIndex, Offset)>,
    ) -> Vec<u8> {
        let mut all_codes = Vec::new();

        let length = self.get_all_byte_length() as usize;

        // jump系命令がラベルの前に存在した場合
        if let Some(tup) = jump_map.get_mut(&group.label) {
            // ラベルまでのバイト数 - ジャンプの位置 - 1 => 相対オフセット
            tup.1 = length as isize - tup.1 - 1;

        } else {
            // ラベルがjump系命令の前に存在した場合
            if !group.label.starts_with("entry_") {
                jump_map.insert(group.label.to_string(), (0, length as isize));
            }
        }


        for i in group.insts.iter() {
            let mut codes = self.generate_from_inst(i, sym_name, jump_map);

            // call命令のオフセットを計算するため，
            // コード長は命令ごとに更新しておく．
            self.add_byte_length(codes.len() as u64);

            all_codes.append(&mut codes);
        }

        all_codes
    }

    fn generate_from_inst(
        &mut self,
        inst: &x64_asm::Instruction,
        sym_name: &str,
        jump_map: &mut HashMap<String, (CodeIndex, Offset)>,
    ) -> Vec<u8> {
        match &inst.opcode {
            Opcode::CALLFUNC(func) => {
                let callee_name = func.copy_label();

                // 再配置用にシンボルを定義
                let mut rela: elf_utilities::relocation::Rela64 = Default::default();
                rela.set_addend(-4);

                // 1 -> オペコード分スキップ generate_callrm64を見るとわかる
                let offset_before_call = self.get_all_byte_length();
                rela.set_offset(offset_before_call + 1);

                self.add_relocation_symbol(sym_name.to_string(), callee_name, rela);

                // 適当なアドレスをおいておく
                vec![0xe8, 0x00, 0x00, 0x00, 0x00]
            }
            // lea
            Opcode::LEAR64FROMSTRADDR { r64: _, str_sym, addend } => {
                // 再配置用にシンボルを定義
                let mut rela: elf_utilities::relocation::Rela64 = Default::default();
                rela.set_addend(*addend as i64);
                let offset_before_lea = self.get_all_byte_length();

                // 3 -> immediateまでスキップ generate_lea64を見るとわかる
                rela.set_offset(offset_before_lea + 4);

                // 2 -> .rodataのsymtabインデックス決め打ち
                rela.set_info((2 << 32) + elf_utilities::relocation::R_X86_64_32);

                self.add_relocation_symbol(sym_name.to_string(), str_sym.to_string(), rela);

                let mut base_bytes = inst.to_bytes();
                base_bytes.append(&mut vec![0x25, 0x00, 0x00, 0x00, 0x00]);

                base_bytes
            }
            // jump
            Opcode::JELABEL { label } => {
                let length = (self.get_all_byte_length() + 1) as usize;

                if let Some(tup) = jump_map.get_mut(label) {
                    // ラベルがjump系命令の前に存在した場合
                    tup.0 = length as isize;
                    tup.1 = !(length as isize + 4 - tup.1) + 1;
                } else {
                    // jump系命令がラベルの前に存在した場合
                    jump_map.insert(label.to_string(), (length as isize, length as isize + 3));
                }

                let mut base_bytes = inst.to_bytes();
                base_bytes.append(&mut vec![0x00; 4]);

                base_bytes
            }
            Opcode::JMPLABEL { label } => {
                let length = (self.get_all_byte_length() + 1) as usize;

                if let Some(tup) = jump_map.get_mut(label) {
                    // ラベルがjump系命令の前に存在した場合
                    tup.0 = length as isize;
                    tup.1 = !(length as isize + 4 - tup.1) + 1;
                } else {
                    // jump系命令がラベルの前に存在した場合
                    jump_map.insert(label.to_string(), (length as isize, length  as isize + 3));
                }

                let mut base_bytes = inst.to_bytes();
                base_bytes.append(&mut vec![0x00; 4]);

                base_bytes
            }
            Opcode::COMMENT(_contents) => Vec::new(),
            _ => inst.to_bytes()
        }
    }

    fn resolve_jump_instructions(
        &self,
        bin_sym: &mut arch::x64::BinSymbol,
        jump_map: &HashMap<String, (CodeIndex, Offset)>,
    ) {
        for (_name, (dst, offset)) in jump_map.iter() {
            for (idx, byte) in (*offset as u32).to_le_bytes().iter().enumerate() {
                bin_sym.set_code(idx + *dst as usize, *byte);
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
