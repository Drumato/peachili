use crate::assembler::x64;
use crate::common::arch;

impl x64::Assembler {
    // e.g. cmpq $0, %rax
    // dst-operand -> r/m field in ModR/M and related b-bit in REX
    pub fn generate_cmpregandint64(
        &self,
        value: &arch::x64::Immediate,
        dst: &arch::x64::Reg64,
    ) -> Vec<u8> {
        let dst_expanded_bit = self.rex_prefix_bbit(dst);
        let rex_prefix = arch::x64::REX_PREFIX_BASE | arch::x64::REX_PREFIX_WBIT | dst_expanded_bit;

        let opcode = 0x81;

        // ModR/M (MI だけど /7 なのでマスクする)
        let rm_field = self.modrm_rm_field(dst);
        let modrm_byte = arch::x64::MODRM_REGISTER_REGISTER | rm_field | 0x38;

        let mut code = vec![rex_prefix, opcode, modrm_byte];

        // immediate
        for byte in value.to_le_bytes() {
            code.push(byte);
        }

        code
    }

    // e.g. cmpq %rdx, %rax
    // dst-operand -> r/m field in ModR/M and related b-bit
    // src-operand -> reg field in ModR/M and related r-bit
    pub fn generate_cmpregandreg64(
        &self,
        src: &arch::x64::Reg64,
        dst: &arch::x64::Reg64,
    ) -> Vec<u8> {
        let dst_expanded_bit = self.rex_prefix_bbit(dst);
        let src_expanded_bit = self.rex_prefix_rbit(src);
        let rex_prefix = arch::x64::REX_PREFIX_BASE
            | arch::x64::REX_PREFIX_WBIT
            | dst_expanded_bit
            | src_expanded_bit;

        let opcode = 0x39;

        // ModR/M (MR)
        let rm_field = self.modrm_rm_field(dst);
        let reg_field = self.modrm_reg_field(src);
        let modrm_byte = arch::x64::MODRM_REGISTER_REGISTER | rm_field | reg_field;

        vec![rex_prefix, opcode, modrm_byte]
    }
}
