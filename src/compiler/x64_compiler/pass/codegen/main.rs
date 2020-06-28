use std::collections::BTreeMap;

use x64_asm::{
    Displacement, GeneralPurposeRegister as GPR, Immediate, Instruction as Inst, Opcode, Operand,
};

use crate::common::arch::x64;
use crate::common::{module, option};
use crate::compiler::general::resource as res;

// x64用コード生成
pub fn codegen(
    _build_option: &option::BuildOption,
    functions: BTreeMap<res::PStringId, res::PFunction>,
    module_allocator: &module::ModuleAllocator,
) -> x64::AssemblyFile {
    let mut generator = Generator::new("asm.s".to_string());

    for (_func_name, func) in functions {
        let const_pool = module_allocator
            .get_module_ref(&func.module_id)
            .unwrap()
            .get_const_pool_ref();
        generator.gen_symbol_from_func(func, &const_pool);
        generator.set_label(1);
    }
    generator.give_assembly()
}

struct Generator {
    asm: x64::AssemblyFile,
    sym_idx: usize,
    label: usize,
    total_string_length: u64,
}

impl Generator {
    fn gen_symbol_from_func(&mut self, func: res::PFunction, const_pool: &res::ConstAllocator) {
        let symbol_name = const_pool
            .get(func.get_func_name_id())
            .unwrap()
            .copy_value();
        let is_main_symbol = symbol_name.as_str() == "main";
        let this_sym = x64::Symbol::new(symbol_name);
        self.add_symbol(this_sym);

        // 関数プロローグ
        let stack_offset = func.get_stack_offset();
        self.gen_function_prologue(stack_offset);

        let local_map = func.get_locals();
        let string_map = func.get_strings();

        // 引数がある場合は，所定のスタックオフセットに格納
        for (arg_i, name_id) in func.get_args().iter().enumerate() {
            let arg_reg = Self::caller_reg64(arg_i);

            let arg_var = local_map.get(vec![*name_id].as_slice());
            if arg_var.is_none() {
                panic!("{:?} is not defined", name_id);
            }

            let disp = Displacement::DISP8(-(arg_var.unwrap().get_stack_offset() as i8));

            self.add_i(self.new_i(x64_asm::new_rm64r64!(
                MOVRM64R64,
                self.new_addressing(GPR::RBP, None, Some(disp)),
                arg_reg
            )));
        }

        for st in func.get_statements() {
            self.gen_insts_from_statement(st, local_map, string_map, const_pool);
        }

        // 暗黙的に return 0; を挿入する．
        if is_main_symbol {
            self.add_i(self.new_i(x64_asm::new_rm64imm32!(
                MOVRM64IMM32,
                self.new_gpr_op(GPR::RAX),
                Immediate::I32(0)
            )));
        }

        self.gen_function_epilogue();

        for (contents_id, hash) in string_map.iter() {
            let contents = const_pool.get(*contents_id).unwrap().copy_value();

            self.add_string_to_cursym(contents, *hash);
        }
    }

    fn gen_insts_from_statement(
        &mut self,
        st: &res::StatementNode,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        string_map: &BTreeMap<res::PStringId, u64>,
        const_pool: &res::ConstAllocator,
    ) {
        match &st.kind {
            res::StatementNodeKind::RETURN(expr) => {
                self.gen_return_statement(expr, local_map, string_map, const_pool)
            }
            res::StatementNodeKind::IFRET(expr) => {
                self.gen_ifret_statement(expr, local_map, string_map, const_pool)
            }
            res::StatementNodeKind::EXPR(expr) => {
                self.gen_expression_statement(expr, local_map, string_map, const_pool)
            }
            res::StatementNodeKind::VARDECL => (),
            res::StatementNodeKind::COUNTUP(id, start, end, body) => {
                self.gen_countup_statement(id, start, end, body, local_map, string_map, const_pool)
            }
            res::StatementNodeKind::ASM(args) => {
                for arg_id in args.iter() {
                    let inst = self.gen_inst_from_asm(*arg_id, const_pool);
                    self.add_i(inst);
                }
            }
            res::StatementNodeKind::VARINIT(ident, expr) => {
                self.gen_varinit_statement(ident, expr, local_map, string_map, const_pool)
            }
        }
    }

    fn gen_return_statement(
        &mut self,
        expr: &res::ExpressionNode,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        string_map: &BTreeMap<res::PStringId, u64>,
        const_pool: &res::ConstAllocator,
    ) {
        self.gen_comment("start return statement");

        self.gen_expr(expr, local_map, string_map, const_pool);
        self.gen_popreg64(GPR::RAX);

        self.gen_comment("end return statement");
    }

    fn gen_ifret_statement(
        &mut self,
        expr: &res::ExpressionNode,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        string_map: &BTreeMap<res::PStringId, u64>,
        const_pool: &res::ConstAllocator,
    ) {
        self.gen_comment("start ifret statement");

        self.gen_expr(expr, local_map, string_map, const_pool);

        self.gen_comment("end ifret statement");
    }

    fn gen_expression_statement(
        &mut self,
        expr: &res::ExpressionNode,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        string_map: &BTreeMap<res::PStringId, u64>,
        const_pool: &res::ConstAllocator,
    ) {
        self.gen_comment("start expression statement");

        self.gen_expr(expr, local_map, string_map, const_pool);

        self.gen_comment("end expression statement");
    }

    fn gen_varinit_statement(
        &mut self,
        ident: &res::ExpressionNode,
        expr: &res::ExpressionNode,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        string_map: &BTreeMap<res::PStringId, u64>,
        const_pool: &res::ConstAllocator,
    ) {
        self.gen_comment("start varinit statement");

        self.gen_assignment_left_value(ident, expr, local_map, string_map, const_pool);

        self.gen_comment("end varinit statement");
    }

    #[allow(clippy::too_many_arguments)]
    fn gen_countup_statement(
        &mut self,
        _id: &res::ExpressionNode,
        _start: &res::ExpressionNode,
        _end: &res::ExpressionNode,
        _body: &[res::StatementNode],
        _local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        _string_map: &BTreeMap<res::PStringId, u64>,
        _const_pool: &res::ConstAllocator,
    ) {
        unimplemented!()
    }

    fn gen_expr(
        &mut self,
        ex: &res::ExpressionNode,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        string_map: &BTreeMap<res::PStringId, u64>,
        const_pool: &res::ConstAllocator,
    ) {
        match &ex.kind {
            // primary
            res::ExpressionNodeKind::STRLIT(contents_id, hash) => {
                // leaq .LS, %rax
                self.add_i(self.new_i(Opcode::LEAR64FROMSTRADDR {
                    r64: GPR::RAX,
                    str_sym: format!(".LS{}", hash),
                    addend: self.total_string_length as usize,
                }));

                let contents = const_pool.get(*contents_id).unwrap();

                // +1 は ヌルバイトを意味する
                self.total_string_length += contents.len() as u64 + 1;

                self.gen_pushreg64(GPR::RAX);
            }
            res::ExpressionNodeKind::TRUE => {
                self.add_i(self.new_i(x64_asm::new_imm32!(PUSHIMM32, Immediate::I32(1))));
            }
            res::ExpressionNodeKind::FALSE => {
                self.add_i(self.new_i(x64_asm::new_imm32!(PUSHIMM32, Immediate::I32(0))));
            }
            res::ExpressionNodeKind::INTEGER(v) => {
                self.add_i(self.new_i(x64_asm::new_imm32!(PUSHIMM32, Immediate::I32(*v as i32))));
            }
            res::ExpressionNodeKind::UNSIGNEDINTEGER(v) => {
                self.add_i(self.new_i(x64_asm::new_imm32!(PUSHIMM32, Immediate::I32(*v as i32))));
            }
            res::ExpressionNodeKind::IDENT(_name) => {
                self.gen_comment("start ident-expression");

                // 変数のアドレスをRAXに
                self.gen_left_value(ex, local_map, string_map);
                self.gen_popreg64(GPR::RAX);

                // RAXをデリファレンス
                self.add_i(self.new_i(x64_asm::new_r64rm64!(
                    MOVR64RM64,
                    GPR::RAX,
                    self.new_mem(GPR::RAX)
                )));
                self.gen_pushreg64(GPR::RAX);

                self.gen_comment("end ident-expression");
            }
            res::ExpressionNodeKind::CALL(ident, args) => {
                self.gen_comment("start call expression");

                // 引数をすべてコンパイル
                for arg in args.iter() {
                    self.gen_expr(arg, local_map, string_map, const_pool);
                }

                let arg_number: usize = if args.is_empty() { 0 } else { args.len() - 1 };

                // スタックに積まれている引数たちを順にレジスタに渡す
                for i in 0..args.len() {
                    let arg_reg = Self::caller_reg64(arg_number - i);
                    self.gen_popreg64(arg_reg);
                }

                let last_name_id = res::IdentName::last_name(ident);
                self.add_i(Inst {
                    opcode: Opcode::CALLFUNC(Operand::LABEL(
                        const_pool.get(last_name_id).unwrap().copy_value(),
                    )),
                });

                self.gen_pushreg64(GPR::RAX);

                self.gen_comment("end call expression");
            }

            // unary-expression
            res::ExpressionNodeKind::NEG(value) => {
                self.gen_unary_expr("-", value, local_map, string_map, const_pool);
            }

            // binary-expression
            res::ExpressionNodeKind::ADD(lop, rop) => {
                self.gen_binary_expr("+", lop, rop, local_map, string_map, const_pool)
            }
            res::ExpressionNodeKind::SUB(lop, rop) => {
                self.gen_binary_expr("-", lop, rop, local_map, string_map, const_pool)
            }
            res::ExpressionNodeKind::MUL(lop, rop) => {
                self.gen_binary_expr("*", lop, rop, local_map, string_map, const_pool)
            }
            res::ExpressionNodeKind::DIV(lop, rop) => {
                self.gen_binary_expr("/", lop, rop, local_map, string_map, const_pool)
            }
            res::ExpressionNodeKind::ASSIGN(lval, rval) => {
                self.gen_comment("start assign expression");

                self.gen_assignment_left_value(lval, rval, local_map, string_map, const_pool);

                self.gen_comment("end assign expression");
            }
            res::ExpressionNodeKind::IF(condition, body) => {
                self.gen_comment("start if expression");

                self.gen_expr(condition, local_map, string_map, const_pool);
                let fin_label = format!(".Lend{}", self.consume_label());

                // condition
                self.gen_popreg64(GPR::RAX);
                self.add_i(self.new_i(Opcode::CMPRAXIMM32 {
                    imm: Immediate::I32(0),
                }));
                self.add_i(self.new_i(Opcode::JELABEL {
                    label: fin_label.clone(),
                }));

                for st in body.iter() {
                    self.gen_insts_from_statement(st, local_map, string_map, const_pool);
                }

                self.add_group_to_cursym(fin_label);

                self.gen_comment("end if expression");
            }
            res::ExpressionNodeKind::IFELSE(condition, body, alter) => {
                self.gen_comment("start if-else expression");

                self.gen_expr(condition, local_map, string_map, const_pool);
                let label_num = self.consume_label();
                let else_label = format!(".Lelse{}", label_num);
                let fin_label = format!(".Lend{}", label_num);

                // condition
                self.gen_popreg64(GPR::RAX);
                self.add_i(self.new_i(Opcode::CMPRAXIMM32 {
                    imm: Immediate::I32(0),
                }));
                self.add_i(self.new_i(Opcode::JELABEL {
                    label: fin_label.clone(),
                }));

                for st in body.iter() {
                    self.gen_insts_from_statement(st, local_map, string_map, const_pool);
                }

                self.add_i(self.new_i(Opcode::JMPLABEL {
                    label: fin_label.clone(),
                }));
                self.add_group_to_cursym(else_label);

                for st in alter.iter() {
                    self.gen_insts_from_statement(st, local_map, string_map, const_pool);
                }

                self.add_group_to_cursym(fin_label);

                self.gen_comment("end if-else expression");
            }
            _ => unimplemented!(),
        }
    }

    pub fn gen_unary_expr(
        &mut self,
        operator: &str,
        value: &res::ExpressionNode,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        string_map: &BTreeMap<res::PStringId, u64>,
        const_pool: &res::ConstAllocator,
    ) {
        // 1． 子ノードをコンパイル
        self.gen_expr(value, local_map, string_map, const_pool);

        // 2．演算に必要なオペランドをレジスタに取り出す
        self.gen_popreg64(GPR::RAX);

        // 3．各演算に対応する命令を生成する
        match operator {
            "-" => self.add_i(self.new_i(x64_asm::new_rm64!(NEGRM64, self.new_gpr_op(GPR::RAX)))),
            _ => panic!("unsupported operator -> {}", operator),
        }

        // 4．演算結果をスタックに格納
        self.gen_pushreg64(GPR::RAX);
    }

    fn gen_binary_expr(
        &mut self,
        operator: &str,
        lop: &res::ExpressionNode,
        rop: &res::ExpressionNode,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        string_map: &BTreeMap<res::PStringId, u64>,
        const_pool: &res::ConstAllocator,
    ) {
        // 1． 左右子ノードをコンパイル
        self.gen_expr(lop, local_map, string_map, const_pool);
        self.gen_expr(rop, local_map, string_map, const_pool);

        // 2．演算に必要なオペランドをレジスタに取り出す
        self.gen_popreg64(GPR::RDI);
        self.gen_popreg64(GPR::RAX);

        // 3．各演算に対応する命令を生成する
        let inst = Inst {
            opcode: match operator {
                "+" => {
                    x64_asm::new_r64rm64!(ADDR64RM64, GPR::RAX, Operand::GENERALREGISTER(GPR::RDI))
                }
                "-" => {
                    x64_asm::new_r64rm64!(SUBR64RM64, GPR::RAX, Operand::GENERALREGISTER(GPR::RDI))
                }
                "*" => {
                    x64_asm::new_r64rm64!(IMULR64RM64, GPR::RAX, Operand::GENERALREGISTER(GPR::RDI))
                }
                "/" => {
                    self.add_i(Inst {
                        opcode: Opcode::CDQ,
                    });
                    x64_asm::new_rm64!(IDIVRM64, Operand::GENERALREGISTER(GPR::RDI))
                }
                _ => panic!("unsupported operator -> {}", operator),
            },
        };
        self.add_i(inst);

        // 4．演算結果をスタックに格納
        self.gen_pushreg64(GPR::RAX);
    }

    fn gen_function_prologue(&mut self, offset: usize) {
        // save rbp
        self.gen_pushreg64(GPR::RBP);
        self.add_i(Inst {
            opcode: x64_asm::new_r64rm64!(MOVR64RM64, GPR::RBP, Operand::GENERALREGISTER(GPR::RSP)),
        });

        // allocating memory area for auto-var
        if offset != 0 {
            let offset = Immediate::I32(!7 & (offset + 7) as i32);
            self.add_i(self.new_i(x64_asm::new_rm64imm32!(
                SUBRM64IMM32,
                self.new_gpr_op(GPR::RSP),
                offset
            )));
        }
    }

    fn gen_function_epilogue(&mut self) {
        self.add_i(self.new_i(x64_asm::new_rm64r64!(
            MOVRM64R64,
            self.new_gpr_op(GPR::RSP),
            GPR::RBP
        )));
        self.gen_popreg64(GPR::RBP);
        self.add_i(self.new_i(Opcode::RET));
    }

    fn gen_assignment_left_value(
        &mut self,
        lval: &res::ExpressionNode,
        rval: &res::ExpressionNode,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        string_map: &BTreeMap<res::PStringId, u64>,
        const_pool: &res::ConstAllocator,
    ) {
        // 1． 左右子ノードをコンパイル
        //     左辺値はアドレスを生成し，スタックに積んでおく．
        self.gen_left_value(lval, local_map, string_map);
        self.gen_expr(rval, local_map, string_map, const_pool);

        // 2．演算に必要なオペランドをレジスタに取り出す
        self.gen_popreg64(GPR::RDI);
        self.gen_popreg64(GPR::RAX);

        // 3．代入 == メモリに格納
        self.add_i(self.new_i(x64_asm::new_rm64r64!(
            MOVRM64R64,
            self.new_mem(GPR::RAX),
            GPR::RDI
        )));

        // 4. 代入式のため，スタックにRDIの値を積んでおく
        self.gen_pushreg64(GPR::RDI);
    }

    fn gen_left_value(
        &mut self,
        lval: &res::ExpressionNode,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        _string_map: &BTreeMap<res::PStringId, u64>,
    ) {
        match &lval.kind {
            res::ExpressionNodeKind::IDENT(id_name) => {
                let name_id = res::IdentName::last_name(id_name);
                let cur_pvar = local_map.get(vec![name_id].as_slice());

                if cur_pvar.is_none() {
                    panic!("{:?} is not defined", name_id);
                }

                self.add_i(self.new_i(x64_asm::new_rm64r64!(
                    MOVRM64R64,
                    self.new_gpr_op(GPR::RAX),
                    GPR::RBP
                )));
                let offset = Immediate::I32(cur_pvar.unwrap().get_stack_offset() as i32);
                self.add_i(self.new_i(x64_asm::new_rm64imm32!(
                    SUBRM64IMM32,
                    self.new_gpr_op(GPR::RAX),
                    offset
                )));
                self.gen_pushreg64(GPR::RAX);
            }

            res::ExpressionNodeKind::DEREF(ident_ex) => {
                self.gen_left_value(ident_ex, local_map, _string_map);

                // get value from address
                self.gen_popreg64(GPR::RAX);

                self.add_i(self.new_i(x64_asm::new_r64rm64!(
                    MOVR64RM64,
                    GPR::RAX,
                    self.new_mem(GPR::RAX)
                )));
                self.gen_pushreg64(GPR::RAX);
            }
            _ => panic!("can't generate {} as lvalue", lval.kind),
        }
    }

    fn gen_comment(&mut self, contents: &str) {
        self.add_i(self.new_i(Opcode::COMMENT(contents.to_string())));
    }

    fn gen_inst_from_asm(
        &self,
        asm_str_id: res::PStringId,
        const_pool: &res::ConstAllocator,
    ) -> Inst {
        let asm_str = const_pool.get(asm_str_id).unwrap().copy_value();

        let asm_splitted: Vec<&str> = asm_str.split(' ').collect();
        let length = asm_splitted.len();

        match length {
            1 => {
                let inst_name = asm_splitted[0];
                match inst_name {
                    "syscall" => self.new_i(Opcode::SYSCALL),
                    _ => panic!("unable to generate from {}", inst_name),
                }
            }
            2 => {
                let inst_name = asm_splitted[0];
                match inst_name {
                    "call" => self.new_i(Opcode::CALLFUNC(Operand::LABEL(
                        asm_splitted[1].to_string(),
                    ))),
                    _ => panic!("unable to generate from {}", inst_name),
                }
            }
            3 => {
                let inst_name = asm_splitted[0];
                match inst_name {
                    "movq" => {
                        let src_str = if asm_splitted[1].contains(',') {
                            let operand_length = asm_splitted[1].len();
                            &asm_splitted[1][..(operand_length - 1)]
                        } else {
                            asm_splitted[1]
                        };

                        // 数値としてパースできる -> movq $1, %rax 的なの
                        // そうではない           -> movq %rax, %rbx
                        match src_str[1..].parse::<i32>() {
                            Ok(value) => {
                                let reg = GPR::from_at_string(asm_splitted[2]);

                                let imm = Immediate::I32(value);
                                self.new_i(x64_asm::new_rm64imm32!(
                                    MOVRM64IMM32,
                                    self.new_gpr_op(reg),
                                    imm
                                ))
                            }
                            Err(_e) => {
                                let src_reg = GPR::from_at_string(src_str);
                                let dst_reg = GPR::from_at_string(asm_splitted[2]);

                                self.new_i(x64_asm::new_rm64r64!(
                                    MOVRM64R64,
                                    self.new_gpr_op(src_reg),
                                    dst_reg
                                ))
                            }
                        }
                    }
                    _ => panic!("unable to generate from {}", inst_name),
                }
            }
            _ => panic!("unable to generate from {}", asm_str),
        }
    }

    fn gen_pushreg64(&mut self, r: GPR) {
        self.add_i(self.new_i(x64_asm::new_r64!(PUSHR64, r)))
    }
    fn gen_popreg64(&mut self, r: GPR) {
        self.add_i(self.new_i(x64_asm::new_r64!(POPR64, r)))
    }

    fn new(file_path: String) -> Self {
        Self {
            sym_idx: 0,
            asm: x64::AssemblyFile::new(file_path),
            label: 1,
            total_string_length: 0,
        }
    }

    fn set_label(&mut self, lnum: usize) {
        self.label = lnum;
    }

    fn consume_label(&mut self) -> usize {
        let cur_num = self.label;
        self.label += 1;
        cur_num
    }

    fn give_assembly(self) -> x64::AssemblyFile {
        self.asm
    }

    fn condition_symidx(&mut self) {
        self.sym_idx = self.asm.symbols_number() - 1;
    }

    fn add_symbol(&mut self, sym: x64::Symbol) {
        self.asm.add_symbol(sym);
        self.condition_symidx();
    }

    fn add_group_to_cursym(&mut self, label: String) {
        self.asm.add_group_to_sym(self.sym_idx, label);
    }

    fn add_i(&mut self, inst: Inst) {
        self.asm.add_inst_to_sym(self.sym_idx, inst);
    }

    fn add_string_to_cursym(&mut self, string: String, hash: u64) {
        self.asm.add_string_to_sym(self.sym_idx, string, hash);
    }

    fn caller_reg64(idx: usize) -> GPR {
        let regs = vec![GPR::RDI, GPR::RSI, GPR::RDX, GPR::RCX, GPR::R8, GPR::R9];
        regs[idx].clone()
    }

    fn new_i(&self, opcode: Opcode) -> Inst {
        Inst { opcode }
    }
    fn new_addressing(&self, base: GPR, index: Option<GPR>, disp: Option<Displacement>) -> Operand {
        Operand::ADDRESSING {
            base_reg: base,
            index_reg: index,
            displacement: disp,
            scale: None,
        }
    }
    fn new_mem(&self, base: GPR) -> Operand {
        self.new_addressing(base, None, None)
    }
    fn new_gpr_op(&self, r: GPR) -> Operand {
        Operand::GENERALREGISTER(r)
    }
}
