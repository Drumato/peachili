use crate::assembler::x64;
use crate::common::arch;

impl x64::Assembler {
    // e.g. subq %rax, %r15
    // dst-operand -> r/m field in ModR/M and related b-bit
    // src-operand -> reg field in ModR/M and related r-bit
    pub fn generate_subregtoreg64(
        &self,
        src: &arch::x64::Reg64,
        dst: &arch::x64::Reg64,
    ) -> Vec<u8> {
        // rex-prefix
        let dst_expanded_bit = self.rex_prefix_bbit(dst);
        let src_expanded_bit = self.rex_prefix_rbit(src);
        let rex_prefix = arch::x64::REX_PREFIX_BASE
            | arch::x64::REX_PREFIX_WBIT
            | dst_expanded_bit
            | src_expanded_bit;

        // opcode
        let opcode = 0x29;

        // ModR/M(MR)
        let rm_field = self.modrm_rm_field(dst);
        let reg_field = self.modrm_reg_field(src);
        let modrm_byte = arch::x64::MODRM_REGISTER_REGISTER | rm_field | reg_field;

        vec![rex_prefix, opcode, modrm_byte]
    }

    // e.g. subq $24, %rsp
    // dst-operand -> r/m field in ModR/M and related b-bit in REX
    pub fn generate_subregbyuint64(
        &self,
        imm: &arch::x64::Immediate,
        dst: &arch::x64::Reg64,
    ) -> Vec<u8> {
        // rex-prefix
        let dst_expanded_bit = self.rex_prefix_bbit(dst);
        let rex_prefix = arch::x64::REX_PREFIX_BASE | arch::x64::REX_PREFIX_WBIT | dst_expanded_bit;

        // opcode
        let opcode = 0x81;

        // ModR/M(MI) だけど/5なのでマスク
        let rm_field = self.modrm_rm_field(dst);
        let modrm_byte = arch::x64::MODRM_REGISTER_REGISTER | rm_field | 0x28;

        let mut codes = vec![rex_prefix, opcode, modrm_byte];

        // immediate
        for byte in imm.to_le_bytes() {
            codes.push(byte);
        }

        codes
    }
}
