extern crate elf_utilities;

use crate::assembler::x64;
use crate::common::*;

pub fn x64_assemble(build_option: &option::BuildOption, asm_file: arch::x64::AssemblyFile) {
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

    if build_option.stop_link {
        // オブジェクトファイルを生成して終了
        object_file_builder.generate_elf_file("obj.o");
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
