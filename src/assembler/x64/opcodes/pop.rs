use crate::assembler::x64;
use crate::common::arch;

impl x64::Assembler {
    pub fn generate_popreg64(&self, reg: &arch::x64::Reg64) -> Vec<u8> {
        // opcode
        let base_byte = 0x58;
        let modrm_rm_field = self.modrm_rm_field(reg);
        vec![base_byte | modrm_rm_field]
    }
}
