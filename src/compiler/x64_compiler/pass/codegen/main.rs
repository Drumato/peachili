use std::collections::BTreeMap;

use x64_asm::{
    GeneralPurposeRegister as GPR,
    Instruction as Inst,
    Operand,
    Opcode,
    Immediate,
    Displacement,
};

use crate::common::{module, option};
use crate::common::arch::x64;
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
            self.add_inst_to_cursym(Inst {
                opcode: Opcode::MOVRM64R64 {
                    rm64: Operand::ADDRESSING {
                        base_reg: GPR::RBP,
                        index_reg: None,
                        displacement: Some(Displacement::DISP8(-(arg_var.unwrap().get_stack_offset() as i8))),
                        scale: None,
                    },
                    r64: arg_reg,
                }
            });
        }

        for st in func.get_statements() {
            self.gen_insts_from_statement(st, local_map, string_map, const_pool);
        }

        // 暗黙的に return 0; を挿入する．
        if is_main_symbol {
            self.add_inst_to_cursym(Inst {
                opcode: Opcode::MOVRM64IMM32 {
                    rm64: Operand::GENERALREGISTER(GPR::RAX),
                    imm: Immediate::I32(0),
                }
            });
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
                    self.add_inst_to_cursym(inst);
                    // self.add_inst_to_cursym(x64::Instruction::inline_asm(arg.clone()));
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
        id: &res::ExpressionNode,
        start: &res::ExpressionNode,
        end: &res::ExpressionNode,
        body: &[res::StatementNode],
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
        string_map: &BTreeMap<res::PStringId, u64>,
        const_pool: &res::ConstAllocator,
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
                self.add_inst_to_cursym(
                    Inst {
                        opcode: Opcode::LEAR64FROMSTRADDR {
                            r64: GPR::RAX,
                            str_sym: format!(".LS{}", hash),
                            addend: self.total_string_length as usize,
                        }
                    }
                );

                let contents = const_pool.get(*contents_id).unwrap();

                // +1 は ヌルバイトを意味する
                self.total_string_length += contents.len() as u64 + 1;

                self.gen_pushreg64(GPR::RAX);
            }
            res::ExpressionNodeKind::TRUE => {
                self.add_inst_to_cursym(Inst {
                    opcode: Opcode::PUSHIMM32 {
                        imm: Immediate::I32(1),
                    }
                });
            }
            res::ExpressionNodeKind::FALSE => {
                self.add_inst_to_cursym(Inst {
                    opcode: Opcode::PUSHIMM32 {
                        imm: Immediate::I32(0),
                    }
                });
            }
            res::ExpressionNodeKind::INTEGER(v) => {
                self.add_inst_to_cursym(
                    Inst {
                        opcode: Opcode::PUSHIMM32 {
                            imm: Immediate::I32(*v as i32),
                        }
                    }
                );
            }
            res::ExpressionNodeKind::UNSIGNEDINTEGER(v) => {
                self.add_inst_to_cursym(
                    Inst {
                        opcode: Opcode::PUSHIMM32 {
                            imm: Immediate::I32(*v as i32),
                        }
                    }
                );
            }
            res::ExpressionNodeKind::IDENT(name) => {
                self.gen_comment("start ident-expression");

                // 変数のアドレスをRAXに
                self.gen_left_value(ex, local_map, string_map);
                self.gen_popreg64(GPR::RAX);

                // RAXをデリファレンス
                self.add_inst_to_cursym(
                    Inst {
                        opcode: Opcode::MOVR64RM64 {
                            r64: GPR::RAX,
                            rm64: Operand::ADDRESSING {
                                base_reg: GPR::RAX,
                                index_reg: None,
                                scale: None,
                                displacement: None,
                            },
                        },
                    }
                );
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
                self.add_inst_to_cursym(
                    Inst {
                        opcode: Opcode::CALLFUNC(
                            Operand::LABEL(const_pool.get(last_name_id).unwrap().copy_value())
                        )
                    }
                );

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
            "-" => {
                self.add_inst_to_cursym(Inst {
                    opcode: Opcode::NEGRM64 {
                        rm64: Operand::GENERALREGISTER(GPR::RAX),
                    }
                })
            }
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
        match operator {
            "+" => {
                self.add_inst_to_cursym(Inst {
                    opcode: Opcode::ADDR64RM64 {
                        r64: GPR::RAX,
                        rm64: Operand::GENERALREGISTER(GPR::RDI),
                    }
                })
            }
            "-" => {
                self.add_inst_to_cursym(Inst {
                    opcode: Opcode::SUBR64RM64 {
                        r64: GPR::RAX,
                        rm64: Operand::GENERALREGISTER(GPR::RDI),
                    }
                })
            }
            "*" => {
                self.add_inst_to_cursym(Inst {
                    opcode: Opcode::IMULR64RM64 {
                        r64: GPR::RAX,
                        rm64: Operand::GENERALREGISTER(GPR::RDI),
                    }
                })
            }
            "/" => {
                self.add_inst_to_cursym(
                    Inst {
                        opcode: Opcode::CDQ,
                    }
                );
                self.add_inst_to_cursym(Inst {
                    opcode: Opcode::IDIVRM64 {
                        rm64: Operand::GENERALREGISTER(GPR::RDI),
                    }
                });
            }
            _ => panic!("unsupported operator -> {}", operator),
        }

        // 4．演算結果をスタックに格納
        self.gen_pushreg64(GPR::RAX);
    }

    fn gen_function_prologue(&mut self, offset: usize) {
        // save rbp
        self.gen_pushreg64(GPR::RBP);
        self.add_inst_to_cursym(
            Inst {
                opcode: Opcode::MOVR64RM64 {
                    r64: GPR::RBP,
                    rm64: Operand::GENERALREGISTER(GPR::RSP),
                }
            }
        );

        // allocating memory area for auto-var
        if offset != 0 {
            self.add_inst_to_cursym(
                Inst {
                    opcode: Opcode::SUBRM64IMM32 {
                        rm64: Operand::GENERALREGISTER(GPR::RSP),
                        imm: Immediate::I32(!7 & (offset + 7) as i32),
                    }
                }
            );
        }
    }

    fn gen_function_epilogue(&mut self) {
        self.add_inst_to_cursym(
            Inst {
                opcode: Opcode::MOVRM64R64 {
                    rm64: Operand::GENERALREGISTER(GPR::RSP),
                    r64: GPR::RBP,
                }
            }
        );
        self.gen_popreg64(GPR::RBP);
        self.add_inst_to_cursym(
            Inst {
                opcode: Opcode::RET,
            }
        );
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
        self.add_inst_to_cursym(
            Inst {
                opcode: Opcode::MOVRM64R64 {
                    rm64: Operand::ADDRESSING {
                        base_reg: GPR::RAX,
                        index_reg: None,
                        displacement: None,
                        scale: None,
                    },
                    r64: GPR::RDI,
                }
            }
        );


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

                self.add_inst_to_cursym(Inst {
                    opcode: Opcode::MOVRM64R64 {
                        rm64: Operand::GENERALREGISTER(GPR::RAX),
                        r64: GPR::RBP,
                    }
                });
                self.add_inst_to_cursym(
                    Inst {
                        opcode: Opcode::SUBRM64IMM32 {
                            rm64: Operand::GENERALREGISTER(GPR::RAX),
                            imm: Immediate::I32(cur_pvar.unwrap().get_stack_offset() as i32),
                        }
                    }
                );
                self.gen_pushreg64(GPR::RAX);
            }

            res::ExpressionNodeKind::DEREF(ident_ex) => {
                self.gen_left_value(ident_ex, local_map, _string_map);

                // get value from address
                self.gen_popreg64(GPR::RAX);

                self.add_inst_to_cursym(Inst {
                    opcode: Opcode::MOVR64RM64 {
                        r64: GPR::RAX,
                        rm64: Operand::ADDRESSING {
                            base_reg: GPR::RAX,
                            index_reg: None,
                            displacement: None,
                            scale: None,
                        },
                    }
                });
                self.gen_pushreg64(GPR::RAX);
            }
            _ => panic!("can't generate {} as lvalue", lval.kind),
        }
    }

    fn gen_comment(&mut self, contents: &str) {
        self.add_inst_to_cursym(Inst {
            opcode: Opcode::COMMENT(contents.to_string())
        });
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
                    "syscall" => Inst {
                        opcode: Opcode::SYSCALL,
                    },
                    _ => panic!("unable to generate from {}", inst_name),
                }
            }
            2 => {
                let inst_name = asm_splitted[0];
                match inst_name {
                    "call" => Inst {
                        opcode: Opcode::CALLFUNC(Operand::LABEL(asm_splitted[1].to_string())),
                    },
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

                                Inst {
                                    opcode: Opcode::MOVRM64IMM32 {
                                        rm64: Operand::GENERALREGISTER(reg),
                                        imm: Immediate::I32(value),
                                    }
                                }
                            }
                            Err(_e) => {
                                let src_reg = GPR::from_at_string(src_str);
                                let dst_reg = GPR::from_at_string(asm_splitted[2]);

                                Inst {
                                    opcode: Opcode::MOVRM64R64 {
                                        rm64: Operand::GENERALREGISTER(src_reg),
                                        r64: dst_reg,
                                    }
                                }
                            }
                            _ => unimplemented!()
                        }
                    }
                    _ => panic!("unable to generate from {}", inst_name),
                }
            }
            _ => panic!("unable to generate from {}", asm_str),
        }
    }

    fn gen_pushreg64(&mut self, r: GPR) {
        self.add_inst_to_cursym(
            Inst {
                opcode: Opcode::PUSHR64 {
                    r64: r,
                }
            }
        )
    }
    fn gen_popreg64(&mut self, r: GPR) {
        self.add_inst_to_cursym(
            Inst {
                opcode: Opcode::POPR64 {
                    r64: r,
                }
            }
        )
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

    fn add_inst_to_cursym(&mut self, inst: Inst) {
        self.asm.add_inst_to_sym(self.sym_idx, inst);
    }

    fn add_string_to_cursym(&mut self, string: String, hash: u64) {
        self.asm.add_string_to_sym(self.sym_idx, string, hash);
    }

    fn caller_reg64(idx: usize) -> GPR {
        let regs = vec![
            GPR::RDI,
            GPR::RSI,
            GPR::RDX,
            GPR::RCX,
            GPR::R8,
            GPR::R9,
        ];
        regs[idx].clone()
    }
}
