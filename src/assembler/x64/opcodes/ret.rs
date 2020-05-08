use crate::assembler::x64;

impl x64::Assembler {
    pub fn generate_ret(&self) -> Vec<u8> {
        // opcode
        vec![0xc3]
    }
}
