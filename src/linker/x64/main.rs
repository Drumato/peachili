use crate::common::{arch, option};

use crate::linker::x64::StaticLinker;

pub fn x64_static_link(_build_opt: &option::BuildOption, builder: arch::x64::ELFBuilder) -> arch::x64::ELFBuilder {
    let obj_file = builder.give_file();

    let mut static_linker = StaticLinker::new(obj_file);

    static_linker.init_phdrs();

    // パディングしたのでセクションのオフセットを変更する必要がある
    // この段階で変更するのは，allocate_address_to_symbols() で セクションシンボル.st_valueを更新するため
    static_linker.update_sections_offset();

    // .textセクションをアラインのため0x00で埋める．
    // これはGCCもやっている方法
    static_linker.adding_null_byte_to(0);

    // 実際のリンク
    let start_up_routine_address = static_linker.allocate_address_to_symbols();
    static_linker.update_entry_point(start_up_routine_address);
    static_linker.resolve_relocation_symbols();

    // 次に文字列データ用に，0x00 パディングを行う．
    // 二段階に分けるのは,パディングサイズを正しく計算するため．
    static_linker.adding_null_byte_to_nodata();

    static_linker.update_ehdr();


    arch::x64::ELFBuilder::new(static_linker.give_file())
}