use crate::assembler::x64;

impl x64::Assembler {
    pub fn generate_callrm64(&self) -> Vec<u8> {
        // opcode, address(empty)
        vec![0xe8, 0x00, 0x00, 0x00, 0x00]
    }
}
