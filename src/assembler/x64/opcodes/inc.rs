use crate::assembler::x64;
use crate::common::arch;

impl x64::Assembler {
    // e.g. incq %rax
    // value-operand -> r/m field in ModR/M and related b-bit in REX
    pub fn generate_increg64(&self, reg: &arch::x64::Reg64) -> Vec<u8> {
        let value_expanded_bit = self.rex_prefix_bbit(reg);

        let rex_prefix =
            arch::x64::REX_PREFIX_BASE | arch::x64::REX_PREFIX_WBIT | value_expanded_bit;

        // opcode
        let opcode = 0xff;

        // ModR/M (M)
        let rm_field = self.modrm_rm_field(reg);
        let modrm_byte = arch::x64::MODRM_REGISTER_REGISTER | rm_field;

        vec![rex_prefix, opcode, modrm_byte]
    }
}
