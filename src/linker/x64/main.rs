use crate::common::{arch, option};

use crate::linker::x64::StaticLinker;

pub fn x64_static_link(_build_opt: &option::BuildOption, builder: arch::x64::ELFBuilder) -> arch::x64::ELFBuilder {
    let obj_file = builder.give_file();

    let mut static_linker = StaticLinker::new(obj_file);

    // 各種ヘッダの設定
    static_linker.init_phdr();
    static_linker.update_ehdr();

    // phdrs[0].p_offset の位置までNULLセクションを0x00で埋める
    // これはGCCもやっている方法
    static_linker.adding_null_byte_to_null_section();

    // 実際のリンク
    let start_up_routine_address = static_linker.allocate_address_to_symbols();
    static_linker.update_entry_point(start_up_routine_address);
    static_linker.resolve_relocation_symbols();

    // パディングしたのでセクションのオフセットを変更する必要がある
    static_linker.update_sections_offset();


    arch::x64::ELFBuilder::new(static_linker.give_file())
}