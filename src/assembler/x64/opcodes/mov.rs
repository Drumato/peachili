use crate::assembler::x64;
use crate::common::arch;

impl x64::Assembler {
    // e.g. movq %r15, rax
    // dst-operand -> r/m field in ModR/M and related b-bit
    // src-operand -> reg field in ModR/M and related r-bit
    pub fn generate_movregtoreg64(
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
        let opcode = 0x89;

        // ModR/M(MR)
        let rm_field = self.modrm_rm_field(dst);
        let reg_field = self.modrm_reg_field(src);
        let modrm_byte = arch::x64::MODRM_REGISTER_REGISTER | rm_field | reg_field;

        vec![rex_prefix, opcode, modrm_byte]
    }

    // e.g. movq %rax, -4(%rbp)
    // dst-operand -> r/m field in ModR/M and related b-bit
    // src-operand -> reg field in ModR/M and related r-bit
    pub fn generate_movregtomem64(
        &self,
        src: &arch::x64::Reg64,
        base_reg: &arch::x64::Reg64,
        offset: usize,
    ) -> Vec<u8> {
        // rex-prefix
        let base_expanded_bit = self.rex_prefix_bbit(base_reg);
        let src_expanded_bit = self.rex_prefix_rbit(src);
        let rex_prefix = arch::x64::REX_PREFIX_BASE
            | arch::x64::REX_PREFIX_WBIT
            | base_expanded_bit
            | src_expanded_bit;

        // opcode
        let opcode = 0x89;

        // ModR/M(MR)
        let rm_field = self.modrm_rm_field(base_reg);
        let reg_field = self.modrm_reg_field(src);
        let modrm_byte = arch::x64::MODRM_REGISTER_DISPLACEMENT8 | rm_field | reg_field;

        // offset
        let displacement = (-(offset as isize)) as u8;

        vec![rex_prefix, opcode, modrm_byte, displacement]
    }

    // e.g. movq -4(%rbp), %rax
    // dst-operand -> reg field in ModR/M and related r-bit
    // src-operand -> r/m field in ModR/M and related b-bit
    pub fn generate_movmemtoreg64(
        &self,
        src_reg: &arch::x64::Reg64,
        offset: usize,
        dst: &arch::x64::Reg64,
    ) -> Vec<u8> {
        // rex-prefix
        let dst_expanded_bit = self.rex_prefix_rbit(dst);
        let src_expanded_bit = self.rex_prefix_bbit(src_reg);
        let rex_prefix = arch::x64::REX_PREFIX_BASE
            | arch::x64::REX_PREFIX_WBIT
            | src_expanded_bit
            | dst_expanded_bit;

        // opcode
        let opcode = 0x8b;

        // ModR/M(RM)
        let rm_field = self.modrm_rm_field(src_reg);
        let reg_field = self.modrm_reg_field(dst);
        let modrm_byte = arch::x64::MODRM_REGISTER_DISPLACEMENT8 | rm_field | reg_field;

        // offset
        let displacement = (-(offset as isize)) as u8;

        vec![rex_prefix, opcode, modrm_byte, displacement]
    }

    // e.g. movq $6, %rax
    // dst-operand -> r/m field in ModR/M and related b-bit in REX
    pub fn generate_movimmtoreg64(
        &self,
        imm: &arch::x64::Immediate,
        dst: &arch::x64::Reg64,
    ) -> Vec<u8> {
        // rex-prefix
        let dst_expanded_bit = self.rex_prefix_bbit(dst);
        let rex_prefix = arch::x64::REX_PREFIX_BASE | arch::x64::REX_PREFIX_WBIT | dst_expanded_bit;

        // opcode
        let opcode = 0xc7;

        // ModR/M(MI)
        let rm_field = self.modrm_rm_field(dst);
        let modrm_byte = arch::x64::MODRM_REGISTER_REGISTER | rm_field;

        let mut codes = vec![rex_prefix, opcode, modrm_byte];

        // immediate
        for byte in imm.to_le_bytes() {
            codes.push(byte);
        }

        codes
    }
}
