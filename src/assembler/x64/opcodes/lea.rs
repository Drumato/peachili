use crate::assembler::x64;
use crate::common::arch;

impl x64::Assembler {
    // e.g. leaq .LS0(%rip), %rax
    // dst-operand -> reg field in ModR/M and related r-bit in REX
    // src-operand -> r/m field in ModR/M and related b-bit in REX
    pub fn generate_learegtoregwithrip64(
        &self,
        disp32: &arch::x64::Immediate,
        dst: &arch::x64::Reg64,
    ) -> Vec<u8> {
        let rip_reg = arch::x64::Reg64::RIP;
        let dst_expanded_bit = self.rex_prefix_rbit(dst);
        let src_expanded_bit = self.rex_prefix_bbit(&rip_reg);
        let rex_prefix = arch::x64::REX_PREFIX_BASE
            | arch::x64::REX_PREFIX_WBIT
            | dst_expanded_bit
            | src_expanded_bit;

        let opcode = 0x8d;

        // ModR/M (RM)
        let reg_field = self.modrm_reg_field(dst);
        let rm_field = self.modrm_rm_field(&rip_reg);
        let modrm_byte = arch::x64::MODRM_REGISTER | reg_field | rm_field;

        let mut code = vec![rex_prefix, opcode, modrm_byte];

        for b in disp32.to_le_bytes().iter() {
            code.push(*b);
        }

        code
    }
}
