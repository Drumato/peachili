use crate::assembler::x64;

impl x64::Assembler {
    pub fn generate_syscall(&self) -> Vec<u8> {
        // opcode
        vec![0x0f, 0x05]
    }
}
