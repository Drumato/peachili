use elf_utilities::{segment, header};

const PAGE_SIZE: u64 = 0x1000;
const BASE_ADDRESS: u64 = 0x400000;

pub struct StaticLinker {
    elf_file: elf_utilities::file::ELF64,
}

impl StaticLinker {
    pub fn new(f: elf_utilities::file::ELF64) -> Self {
        Self {
            elf_file: f,
        }
    }

    pub fn init_phdr(&mut self) {
        let mut phdr: segment::Phdr64 = Default::default();

        // 機械語命令 -> PT_LOADに配置
        phdr.set_type(segment::TYPE::LOAD);

        // Linux環境ではページサイズアラインされている必要あり
        phdr.set_offset(PAGE_SIZE);
        phdr.set_align(PAGE_SIZE);

        // 決め打ちしたアドレスにロード
        phdr.set_vaddr(BASE_ADDRESS);
        phdr.set_paddr(BASE_ADDRESS);

        let text_section_opt = self.elf_file.get_section(".text".to_string());

        if text_section_opt.is_none() {
            panic!("not found .text section");
        }

        let text_binary_length = text_section_opt.unwrap().header.get_size();

        // .bssではないので filesz/memsz は同じ
        phdr.set_filesz(text_binary_length);
        phdr.set_memsz(text_binary_length);

        // 全フラグを立てておく
        phdr.set_flags(segment::PF_R | segment::PF_X | segment::PF_W);

        self.elf_file.add_segment(segment::Segment64::new(phdr));
    }

    pub fn update_ehdr(&mut self) {
        let all_section_size = self.elf_file.all_section_size();
        let segment_number = self.elf_file.segment_number();
        let ehdr = self.elf_file.get_ehdr_as_mut();

        ehdr.set_elf_type(header::ELFTYPE::EXEC);

        ehdr.set_phoff(header::Ehdr64::size() as u64);
        ehdr.set_phnum(segment_number as u16);
        ehdr.set_phentsize(segment::Phdr64::size());

        ehdr.set_shoff(PAGE_SIZE + all_section_size);
    }

    pub fn adding_null_byte_to_null_section(&mut self) {
        // 0x00を nullセクションに書き込む
        // null-section-header の値は変えないので,どのセクションにも属さないバイナリを書き込む
        let pht_size = segment::Phdr64::size() * self.elf_file.segment_number() as u16;

        self.elf_file.add_null_bytes_to(0, PAGE_SIZE as usize - header::Ehdr64::size() as usize - pht_size as usize);
    }

    pub fn allocate_address_to_symbols(&mut self) -> elf_utilities::Elf64Addr {
        // プロセスのエントリポイントを取得する
        // symbol.st_value には ファイルオフセットが格納されているので，
        // BASE_ADDRESS + st_value -> メモリ上のアドレス，という感じになる
        let mut ehdr_entry: elf_utilities::Elf64Addr = 0;

        // 各シンボルにアドレスを割り当て
        if let Some(symtab_sct) = self.elf_file.get_section_as_mut(".symtab".to_string()) {
            let mut symbols = symtab_sct.symbols.as_ref().unwrap().clone();

            for sym in symbols.iter_mut() {

                // スタートアップルーチンであればエントリポイントに指定
                if sym.compare_symbol_name("initialize".to_string()) {
                    ehdr_entry = BASE_ADDRESS + sym.get_value();
                }

                // 相対オフセットを追加する
                sym.set_value(sym.get_value() + BASE_ADDRESS);
            }

            symtab_sct.symbols = Some(symbols);
        }

        // update_entry_point() 用に返す
        ehdr_entry
    }

    pub fn resolve_relocation_symbols(&mut self) {
        let symbols = self.elf_file.get_section(".symtab".to_string()).unwrap().symbols.as_ref().unwrap().clone();
        let rela_symbols = self.elf_file.get_section(".rela.text".to_string()).unwrap().rela_symbols.as_ref().unwrap().clone();

        // 各再配置シンボルにアドレスを割り当て
        for rela_sym in rela_symbols.iter() {
            // 文字列データの再配置は飛ばす
            let string_literal =
                (rela_sym.get_info() & elf_utilities::relocation::R_X86_64_PC32) != 0;
            if string_literal {
                continue;
            }

            // TODO: 今はR_X86_64_PC32のみ対応
            // Relaオブジェクトに対応するシンボルテーブルエントリからアドレスを取り出す
            let related_symbol_index = rela_sym.bind() as usize;
            let sym_address = symbols[related_symbol_index].get_value() as i32;
            let relative_offset = sym_address - BASE_ADDRESS as i32 - rela_sym.get_offset() as i32 + rela_sym.get_addend() as i32;

            // アドレスをバイト列に変換,機械語に書き込むことでアドレス解決
            for (idx, b) in relative_offset.to_le_bytes().to_vec().iter().enumerate() {
                if let Some(text_sct) = self.elf_file.get_section_as_mut(".text".to_string()) {
                    text_sct.write_byte_to_index(*b, rela_sym.get_offset() as usize + idx);
                }
            }
        }
    }

    pub fn update_sections_offset(&mut self) {
        for sct in self.elf_file.iter_sections_as_mut() {
            let is_text_sct = sct.name == ".text".to_string();
            let update_offset = PAGE_SIZE - header::Ehdr64::size() as u64 + sct.header.get_offset();
            sct.header.set_offset(update_offset);


            if is_text_sct {
                sct.header.set_addr(BASE_ADDRESS);
            }
        }
    }

    pub fn update_entry_point(&mut self, entry: elf_utilities::Elf64Addr) {
        let ehdr = self.elf_file.get_ehdr_as_mut();
        ehdr.set_entry(entry);
    }

    pub fn give_file(self) -> elf_utilities::file::ELF64 {
        self.elf_file
    }
}