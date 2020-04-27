use std::collections::BTreeMap;

use crate::common::arch;

pub struct Assembler {
    symbol_map: BTreeMap<String, arch::x64::BinSymbol>,
    // _relocation_map: BTreeMap<String, elf_util::reloc::Rela64>,

    // all_bytes : u64,
}

impl Default for Assembler {
    fn default() -> Self {
        Self {
            symbol_map: BTreeMap::new(),
        }
    }
}

impl Assembler {
    pub fn add_symbol(&mut self, name: String, sym: arch::x64::BinSymbol) {
        self.symbol_map.insert(name, sym);
    }

    // ModR/M 関連
    pub fn modrm_rm_field(&self, reg: &arch::x64::Reg64) -> u8 {
        reg.machine_number() << 3
    }
    pub fn modrm_reg_field(&self, reg: &arch::x64::Reg64) -> u8 {
        reg.machine_number()
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
