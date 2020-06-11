extern crate elf_utilities;

use std::io::{BufWriter, Write};
use std::os::unix::fs::OpenOptionsExt;

use crate::assembler::x64;

pub struct ELFBuilder {
    obj_file: elf_utilities::file::ELF64,
}

#[allow(dead_code)]
impl ELFBuilder {
    pub fn new(elf_file: elf_utilities::file::ELF64) -> Self {
        Self { obj_file: elf_file }
    }

    pub fn add_section(&mut self, section: elf_utilities::section::Section64) {
        self.obj_file.add_section(section);
    }

    pub fn give_file(self) -> elf_utilities::file::ELF64 {
        self.obj_file
    }

    pub fn condition_elf_header(&mut self) {
        self.obj_file.condition();
    }

    pub fn generate_elf_file(&self, file_path: &str, mode: u32) {
        let bytes = self.obj_file.to_le_bytes();

        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .mode(mode)
            .open(file_path)
            .unwrap();
        let mut writer = BufWriter::new(file);
        match writer.write_all(&bytes) {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e),
        }
        match writer.flush() {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e),
        }
    }

    pub fn section_number(&self) -> usize {
        self.obj_file.section_number()
    }

    pub fn add_text_section(&mut self, generator: &x64::Assembler) {
        // すべてのシンボルのコードを結合する
        let mut all_machine_codes: Vec<u8> = Vec::new();

        let symbol_map = generator.get_symbol_map();
        for (_name, sym) in symbol_map.iter() {
            let mut symbol_codes = sym.copy_codes();
            all_machine_codes.append(&mut symbol_codes);
        }

        // .textセクションの生成
        let text_shdr =
            self.init_text_section_header(all_machine_codes.len() as elf_utilities::Elf64Xword);
        let mut text_section =
            elf_utilities::section::Section64::new(".text".to_string(), text_shdr);
        text_section.bytes = Some(all_machine_codes);

        self.add_section(text_section);
    }

    pub fn add_symbol_table_section(&mut self, generator: &x64::Assembler) {
        // NULLシンボル + .textシンボル + .rodataシンボル
        let mut elf_symbols = vec![
            elf_utilities::symbol::Symbol64::new_null_symbol(),
            self.create_section_symbol(1),
            self.create_section_symbol(6),
        ];

        // シンボルを走査する
        // name_indexの操作も行う.
        // また,各シンボルのオフセットも計算する.
        let mut symbol_name_index: elf_utilities::Elf64Word = 1; // 最初のnull文字を飛ばす
        let mut symbol_offset: elf_utilities::Elf64Addr = 0; // st_value用

        let symbol_map = generator.get_symbol_map();
        for (symbol_name, symbol_info) in symbol_map.iter() {
            let symbol_code_length = symbol_info.code_length();
            let symbol_name_length = symbol_name.len();

            let mut global_symbol = self.create_global_symbol(
                symbol_name_index,
                symbol_code_length as u64,
                symbol_offset,
            );
            global_symbol.set_symbol_name(symbol_name.to_string());
            elf_symbols.push(global_symbol);

            // シンボル名を指すインデックスの更新( null byte を見越して+1する)
            symbol_name_index += symbol_name_length as elf_utilities::Elf64Word + 1;

            // オフセットの更新
            // 後ろのシンボルのオフセット <- 前のシンボルのサイズの総合値
            symbol_offset += symbol_code_length as elf_utilities::Elf64Addr;
        }

        let symbol_table_size = elf_symbols.len() * elf_utilities::symbol::Symbol64::size() as usize;
        // セクションの追加
        let symtab_section_header =
            self.init_symbol_table_section_header(symbol_table_size as u64);
        let mut symtab_section =
            elf_utilities::section::Section64::new(".symtab".to_string(), symtab_section_header);
        symtab_section.symbols = Some(elf_symbols);
        self.add_section(symtab_section);
    }

    pub fn add_symtab_string_section(&mut self, generator: &x64::Assembler) {
        // シンボルマップをイテレートして,名前を集める.
        let symbol_map = generator.get_symbol_map();
        let symbol_names: Vec<&str> = symbol_map
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<&str>>();

        let symbol_string_table = elf_utilities::section::build_string_table(symbol_names);
        let strtab_header =
            self.init_string_table_header(symbol_string_table.len() as elf_utilities::Elf64Xword);
        let mut strtab_section =
            elf_utilities::section::Section64::new(".strtab".to_string(), strtab_header);
        strtab_section.bytes = Some(symbol_string_table);
        self.add_section(strtab_section);
    }

    pub fn add_relatext_section(&mut self, generator: &x64::Assembler) {
        // BTreeMap<String, Rela64> -> Vec<&Rela64>
        let relocation_map = generator.get_relocation_map();
        let mut rela_vector: Vec<elf_utilities::relocation::Rela64> = Vec::new();

        for (_caller_name, each_sym_rel_map) in relocation_map.iter() {
            for (_callee_name, callee_symbols) in each_sym_rel_map.iter() {
                for callee_symbol in callee_symbols.iter() {
                    rela_vector.push(callee_symbol.clone());
                }
            }
        }

        // Relaオブジェクトをバイナリに変換
        let mut rela_table_binary: Vec<u8> = Vec::new();
        for rela in rela_vector.iter() {
            let mut rela_entry_binary = rela.to_le_bytes();
            rela_table_binary.append(&mut rela_entry_binary);
        }

        let relatext_hdr = self.init_relatext_header(rela_table_binary.len() as u64);
        let mut relatext_section =
            elf_utilities::section::Section64::new(".rela.text".to_string(), relatext_hdr);
        relatext_section.rela_symbols = Some(rela_vector);
        self.add_section(relatext_section);
    }

    pub fn add_nodata_section(&mut self) {
        let nodata_header = self.init_nodata_header();
        let mut nodata_section =
            elf_utilities::section::Section64::new(".nodata".to_string(), nodata_header);
        nodata_section.bytes = Some(Vec::new());
        self.add_section(nodata_section);
    }

    pub fn add_rodata_section(&mut self, generator: &x64::Assembler) {
        let symbol_map = generator.get_symbol_map();

        let strings_each_sym = symbol_map
            .iter()
            .map(|(_, sym)| sym.copy_strings())
            .collect::<Vec<Vec<String>>>();

        let mut strings: Vec<Vec<u8>> = Vec::new();

        for strs in strings_each_sym.iter() {
            for st in strs.iter() {
                strings.push(st.replace("\\n", "\n").as_bytes().to_vec());
            }
        }

        // 文字列リテラルの数
        let strings_number = strings.len() as u64;
        let strtab: Vec<u8> = elf_utilities::section::build_byte_string_table(strings);

        // TODO: 空っぽ

        let rodata_header = self.init_rodata_header(strtab.len() as u64, strings_number);
        let mut rodata_section =
            elf_utilities::section::Section64::new(".rodata".to_string(), rodata_header);
        rodata_section.bytes = Some(strtab);
        self.add_section(rodata_section);
    }

    pub fn add_shstrtab_string_section(&mut self) {
        // TODO: 決め打ち
        let section_names = vec![
            ".text",
            ".symtab",
            ".strtab",
            ".rela.text",
            ".nodata",
            ".rodata",
            ".shstrtab",
        ];

        let section_string_table = elf_utilities::section::build_string_table(section_names);
        let shstrtab_header =
            self.init_string_table_header(section_string_table.len() as elf_utilities::Elf64Xword);
        let mut shstrtab_section =
            elf_utilities::section::Section64::new(".shstrtab".to_string(), shstrtab_header);
        shstrtab_section.bytes = Some(section_string_table);
        self.add_section(shstrtab_section);
    }

    fn init_text_section_header(
        &self,
        length: elf_utilities::Elf64Xword,
    ) -> elf_utilities::section::Shdr64 {
        let mut shdr: elf_utilities::section::Shdr64 = Default::default();

        shdr.set_type(elf_utilities::section::TYPE::PROGBITS);
        shdr.set_size(length);
        shdr.set_addralign(1);
        shdr.set_flags(elf_utilities::section::SHF_ALLOC | elf_utilities::section::SHF_EXECINSTR);

        shdr
    }

    fn init_symbol_table_section_header(
        &self,
        length: elf_utilities::Elf64Xword,
    ) -> elf_utilities::section::Shdr64 {
        let mut shdr: elf_utilities::section::Shdr64 = Default::default();

        shdr.set_type(elf_utilities::section::TYPE::SYMTAB);
        shdr.set_size(length);
        shdr.set_addralign(1);
        shdr.set_entry_size(elf_utilities::symbol::Symbol64::size());

        // TODO: .strtabが3番目にあることを決め打ち
        shdr.set_link(3);

        // TODO: 最初のグローバルシンボルが4番目にあることを決め打ち
        shdr.set_info(3);
        shdr
    }

    fn init_string_table_header(
        &self,
        length: elf_utilities::Elf64Xword,
    ) -> elf_utilities::section::Shdr64 {
        let mut shdr: elf_utilities::section::Shdr64 = Default::default();

        shdr.set_type(elf_utilities::section::TYPE::STRTAB);
        shdr.set_size(length);
        shdr.set_addralign(1);

        shdr
    }

    fn init_relatext_header(
        &self,
        length: elf_utilities::Elf64Xword,
    ) -> elf_utilities::section::Shdr64 {
        let mut shdr: elf_utilities::section::Shdr64 = Default::default();

        shdr.set_type(elf_utilities::section::TYPE::RELA);
        shdr.set_size(length);
        shdr.set_flags(elf_utilities::section::section_flag::SHF_INFO_LINK);
        shdr.set_addralign(8);
        shdr.set_entry_size(elf_utilities::relocation::Rela64::size());

        // TODO: シンボルテーブルが2番目にあることを決め打ち
        shdr.set_link(2);

        // TODO: .textセクションが一番目にあることを決め打ち
        shdr.set_info(1);

        shdr
    }

    fn init_nodata_header(&self) -> elf_utilities::section::Shdr64 {
        let mut shdr: elf_utilities::section::Shdr64 = Default::default();

        shdr.set_type(elf_utilities::section::TYPE::NULL);

        shdr
    }

    fn init_rodata_header(
        &self,
        length: elf_utilities::Elf64Xword,
        string_number: elf_utilities::Elf64Xword,
    ) -> elf_utilities::section::Shdr64 {
        let mut shdr: elf_utilities::section::Shdr64 = Default::default();

        shdr.set_type(elf_utilities::section::TYPE::PROGBITS);
        shdr.set_size(length);
        shdr.set_entry_size(string_number);
        shdr.set_flags(elf_utilities::section::section_flag::SHF_ALLOC);
        shdr.set_addralign(1);

        shdr
    }
    fn create_global_symbol(
        &self,
        st_name: elf_utilities::Elf64Word,
        st_size: elf_utilities::Elf64Xword,
        st_offset: elf_utilities::Elf64Addr,
    ) -> elf_utilities::symbol::Symbol64 {
        let mut symbol: elf_utilities::symbol::Symbol64 = Default::default();
        symbol.set_name(st_name);
        symbol.set_size(st_size);
        symbol.set_value(st_offset);

        // TODO: .textが1番目にあることを決め打ち
        symbol.set_shndx(1);

        // グローバル + Function属性
        let sym_info = elf_utilities::symbol::symbol_info(
            elf_utilities::symbol::STB_GLOBAL,
            elf_utilities::symbol::STT_FUNC,
        );
        symbol.set_info(sym_info);

        symbol
    }

    fn create_section_symbol(&self, shndx: u16) -> elf_utilities::symbol::Symbol64 {
        let mut symbol: elf_utilities::symbol::Symbol64 = Default::default();

        symbol.set_shndx(shndx);

        // ローカル + SECTION属性
        let sym_info = elf_utilities::symbol::symbol_info(
            elf_utilities::symbol::STB_LOCAL,
            elf_utilities::symbol::STT_SECTION,
        );
        symbol.set_info(sym_info);

        symbol
    }
}
