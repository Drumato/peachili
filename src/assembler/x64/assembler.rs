extern crate elf_utilities;
extern crate indexmap;

use std::collections::BTreeMap;

use crate::common::arch;

pub struct Assembler {
    symbol_map: indexmap::IndexMap<String, arch::x64::BinSymbol>,
    // caller_name -> callee_symbol
    relocation_map:
        indexmap::IndexMap<String, BTreeMap<String, Vec<elf_utilities::relocation::Rela64>>>,
    all_bytes: u64,
}

impl Default for Assembler {
    fn default() -> Self {
        Self {
            symbol_map: indexmap::IndexMap::new(),
            relocation_map: indexmap::IndexMap::new(),
            all_bytes: 0,
        }
    }
}

impl Assembler {
    // 再配置情報の更新
    pub fn setup_relocation_informations(&mut self, asm_file: &arch::x64::AssemblyFile) {
        let mut total_offset = 0;

        let symbols = asm_file.get_symbols();
        for symbol_info in symbols.iter() {
            let caller_name = symbol_info.copy_name();
            let symbol_code_length =
                self.symbol_map.get(&caller_name).unwrap().code_length() as u64;

            if let Some(rela_map) = self.relocation_map.get_mut(&caller_name) {
                for (rela_name, relas) in rela_map.iter_mut() {
                    // 同じ関数を複数回呼んでいるケースもあるため
                    for rela in relas.iter_mut() {
                        // シンボル内のオフセット -> コード全体でのオフセット
                        let current_offset = rela.get_offset();
                        rela.set_offset(current_offset + total_offset);

                        // 文字列リテラルは飛ばす
                        let string_literal =
                            (rela.get_info() & elf_utilities::relocation::R_X86_64_PC32) != 0;
                        if string_literal {
                            continue;
                        }

                        // NULL シンボル+セクションシンボルのことを考えて+3する
                        let mut sym_idx = 0;
                        for (idx, sym) in symbols.iter().enumerate() {
                            if sym.copy_name() == *rela_name {
                                sym_idx = idx;
                            }
                        }
                        let skip_null_and_section = (sym_idx + 3) as u64;

                        // シンボルテーブルのインデックスはr_infoのうち上位32bitを使う
                        let shifted_until_symbol_index = skip_null_and_section << 32;

                        rela.set_info(
                            shifted_until_symbol_index + elf_utilities::relocation::R_X86_64_PLT32,
                        );
                    }
                }
            }

            total_offset += symbol_code_length;
        }
    }

    pub fn add_symbol(&mut self, name: String, sym: arch::x64::BinSymbol) {
        self.symbol_map.insert(name, sym);
    }
    pub fn add_relocation_symbol(
        &mut self,
        caller: String,
        callee: String,
        reloc_sym: elf_utilities::relocation::Rela64,
    ) {
        if let Some(map) = self.relocation_map.get_mut(&caller) {
            if let Some(relas) = map.get_mut(&callee) {
                relas.push(reloc_sym);
            } else {
                map.insert(callee, vec![reloc_sym]);
            }
            return;
        }

        self.relocation_map.insert(caller.clone(), BTreeMap::new());

        if let Some(map) = self.relocation_map.get_mut(&caller) {
            map.insert(callee, vec![reloc_sym]);
            return;
        }
    }
    pub fn add_byte_length(&mut self, length: u64) {
        self.all_bytes += length;
    }
    pub fn set_byte_length(&mut self, length: u64) {
        self.all_bytes = length;
    }
    pub fn get_symbol_map(&self) -> &indexmap::IndexMap<String, arch::x64::BinSymbol> {
        &self.symbol_map
    }
    pub fn get_relocation_map(
        &self,
    ) -> &indexmap::IndexMap<String, BTreeMap<String, Vec<elf_utilities::relocation::Rela64>>> {
        &self.relocation_map
    }
    pub fn get_all_byte_length(&self) -> u64 {
        self.all_bytes
    }

    // ModR/M 関連
    pub fn modrm_rm_field(&self, reg: &arch::x64::Reg64) -> u8 {
        reg.machine_number()
    }
    pub fn modrm_reg_field(&self, reg: &arch::x64::Reg64) -> u8 {
        reg.machine_number() << 3
    }

    // REX-Prefix 関連
    pub fn rex_prefix_bbit(&self, reg: &arch::x64::Reg64) -> u8 {
        if reg.is_expanded() {
            arch::x64::REX_PREFIX_RBIT
        } else {
            0x00
        }
    }
    pub fn rex_prefix_rbit(&self, reg: &arch::x64::Reg64) -> u8 {
        if reg.is_expanded() {
            arch::x64::REX_PREFIX_BBIT
        } else {
            0x00
        }
    }
}
