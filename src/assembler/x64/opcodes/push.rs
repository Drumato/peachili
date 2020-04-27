use crate::assembler::x64;
use crate::common::arch;

impl x64::Assembler {
    pub fn generate_pushreg64(&self, reg: &arch::x64::Reg64) -> Vec<u8> {
        // opcode
        let base_byte = 0x50;
        let modrm_rm_field = self.modrm_rm_field(reg);
        vec![base_byte | modrm_rm_field]
    }

    pub fn generate_pushint64(&self, value: &arch::x64::Immediate) -> Vec<u8> {
        // opcode
        let opcode = 0x68;
        let mut codes = vec![opcode];

        // immediate
        for byte in value.to_le_bytes() {
            codes.push(byte);
        }

        codes
    }
}
