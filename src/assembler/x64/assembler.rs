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
}
