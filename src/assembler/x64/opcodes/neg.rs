use crate::assembler::x64;
use crate::common::arch;

impl x64::Assembler {
    // e.g. negq %rax
    // REX.W + 0xf7 /3
    // value-operand -> r/m field in ModR/M and related b-bit in REX
    pub fn generate_negreg64(&self, reg: &arch::x64::Reg64) -> Vec<u8> {
        let value_expanded_bit = self.rex_prefix_bbit(reg);

        let rex_prefix =
            arch::x64::REX_PREFIX_BASE | arch::x64::REX_PREFIX_WBIT | value_expanded_bit;

        // opcode
        let opcode = 0xf7;

        // ModR/M (Mだけど /3 なのでマスクする)
        let rm_field = self.modrm_rm_field(reg);
        let modrm_byte = arch::x64::MODRM_REGISTER_REGISTER | rm_field | 0x18;

        vec![rex_prefix, opcode, modrm_byte]
    }
}
