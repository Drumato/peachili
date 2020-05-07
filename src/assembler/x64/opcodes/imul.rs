use crate::assembler::x64;
use crate::common::arch;

impl x64::Assembler {
    // e.g. imulq %r15, %rax
    // dst-operand -> reg field in ModR/M and related r-bit in REX
    // src1-operand -> r/m field in ModR/M and related b-bit in REX
    pub fn generate_imulregtoreg64(
        &self,
        src: &arch::x64::Reg64,
        dst: &arch::x64::Reg64,
    ) -> Vec<u8> {
        // rex-prefix
        let dst_expanded_bit = self.rex_prefix_rbit(dst);
        let src_expanded_bit = self.rex_prefix_bbit(src);
        let rex_prefix = arch::x64::REX_PREFIX_BASE
            | arch::x64::REX_PREFIX_WBIT
            | dst_expanded_bit
            | src_expanded_bit;

        let opcode1 = 0x0f;
        let opcode2 = 0xaf;

        // ModR/M(RM)
        let rm_field = self.modrm_rm_field(src);
        let reg_field = self.modrm_reg_field(dst);
        let modrm_byte = arch::x64::MODRM_REGISTER_REGISTER | rm_field | reg_field;

        vec![rex_prefix, opcode1, opcode2, modrm_byte]
    }
}
