use std::collections::BTreeMap;

use crate::common::arch::x64;
use crate::common::arch::x64::Reg64;
use crate::common::option;
use crate::compiler::resource as res;

// x64用コード生成
pub fn codegen(
    _build_option: &option::BuildOption,
    functions: Vec<res::PFunction>,
) -> x64::AssemblyFile {
    let mut generator = Generator::new("asm.s".to_string());

    for func in functions {
        generator.gen_symbol_from_func(func);
        generator.set_label(1);
    }
    generator.give_assembly()
}

struct Generator {
    asm: x64::AssemblyFile,
    sym_idx: usize,
    label: usize,
}

impl Generator {
    fn gen_symbol_from_func(&mut self, func: res::PFunction) {
        let symbol_name = func.copy_func_name();
        let this_sym = x64::Symbol::new(symbol_name);
        self.add_symbol(this_sym);

        // 関数プロローグ
        let stack_offset = func.get_stack_offset();
        self.gen_function_prologue(stack_offset);

        let local_map = func.get_locals();

        // 引数がある場合は，所定のスタックオフセットに格納
        for (arg_i, name) in func.get_args().iter().enumerate() {
            let arg_reg = Self::caller_reg64(arg_i);

            let arg_var = local_map.get(name);
            if arg_var.is_none() {
                panic!("{} is not defined", name);
            }
            self.add_inst_to_cursym(x64::Instruction::movreg_tomem64(
                arg_reg,
                Reg64::RBP,
                arg_var.unwrap().get_stack_offset(),
            ));
        }

        for st in func.get_statements() {
            self.gen_insts_from_statement(st, local_map);
        }

        self.gen_function_epilogue();
    }

    fn gen_insts_from_statement(
        &mut self,
        st: &res::StatementNode,
        local_map: &BTreeMap<String, res::PVariable>,
    ) {
        match &st.kind {
            res::StatementNodeKind::RETURN(expr) => self.gen_return_statement(expr, local_map),
            res::StatementNodeKind::IFRET(expr) => self.gen_ifret_statement(expr, local_map),
            res::StatementNodeKind::EXPR(expr) => self.gen_expression_statement(expr, local_map),
            res::StatementNodeKind::VARDECL => (),
            res::StatementNodeKind::COUNTUP(id, start, end, body) => {
                self.gen_countup_statement(id, start, end, body, local_map)
            }
            res::StatementNodeKind::ASM(args) => {
                for arg in args.iter() {
                    self.add_inst_to_cursym(x64::Instruction::inline_asm(arg.clone()));
                }
            }
        }
    }

    fn gen_return_statement(
        &mut self,
        expr: &res::ExpressionNode,
        local_map: &BTreeMap<String, res::PVariable>,
    ) {
        self.gen_comment("start return statement");

        self.gen_expr(expr, local_map);
        self.add_inst_to_cursym(x64::Instruction::popreg64(x64::Reg64::RAX));

        self.gen_comment("end return statement");
    }

    fn gen_ifret_statement(
        &mut self,
        expr: &res::ExpressionNode,
        local_map: &BTreeMap<String, res::PVariable>,
    ) {
        self.gen_comment("start ifret statement");

        self.gen_expr(expr, local_map);

        self.gen_comment("end ifret statement");
    }

    fn gen_expression_statement(
        &mut self,
        expr: &res::ExpressionNode,
        local_map: &BTreeMap<String, res::PVariable>,
    ) {
        self.gen_comment("start expression statement");

        self.gen_expr(expr, local_map);

        self.gen_comment("end expression statement");
    }

    fn gen_countup_statement(
        &mut self,
        id: &res::ExpressionNode,
        start: &res::ExpressionNode,
        end: &res::ExpressionNode,
        body: &Vec<res::StatementNode>,
        local_map: &BTreeMap<String, res::PVariable>,
    ) {
        self.gen_comment("start countup statement");

        let lnum = self.consume_label();
        let start_label = format!(".Lstart{}", lnum);
        let end_label = format!(".Lend{}", lnum);

        // initialize

        self.gen_left_value(id, local_map);
        self.gen_expr(start, local_map);
        self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RDI));
        self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RAX));
        self.add_inst_to_cursym(x64::Instruction::movreg_tomem64(Reg64::RDI, Reg64::RAX, 0));

        // in loop
        self.add_inst_to_cursym(x64::Instruction::label(start_label.clone()));

        // check whether condition is satisfied
        self.gen_expr(id, local_map);
        self.gen_expr(end, local_map);
        self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RDI));
        self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RAX));
        self.add_inst_to_cursym(x64::Instruction::cmpreg_andreg64(Reg64::RDI, Reg64::RAX));
        self.add_inst_to_cursym(x64::Instruction::jump_equal_label(end_label.clone()));

        // contents
        for st in body.iter() {
            self.gen_insts_from_statement(st, local_map);
        }

        // increment
        self.gen_left_value(id, local_map);
        self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RAX));
        self.add_inst_to_cursym(x64::Instruction::movmem_toreg64(Reg64::RAX, 0, Reg64::RDI));
        self.add_inst_to_cursym(x64::Instruction::increg64(Reg64::RDI));
        self.add_inst_to_cursym(x64::Instruction::movreg_tomem64(Reg64::RDI, Reg64::RAX, 0));

        self.add_inst_to_cursym(x64::Instruction::jump_label(start_label));
        self.add_inst_to_cursym(x64::Instruction::label(end_label.clone()));

        self.gen_comment("end countup statement");
    }

    fn gen_expr(&mut self, ex: &res::ExpressionNode, local_map: &BTreeMap<String, res::PVariable>) {
        match &ex.kind {
            // primary
            res::ExpressionNodeKind::INTEGER(v) => {
                self.add_inst_to_cursym(x64::Instruction::pushint64(*v));
            }
            res::ExpressionNodeKind::IDENT(_id_name) => {
                self.gen_comment("start identifier expression");
                self.gen_left_value(ex, local_map);
                self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RAX));

                // get value from address
                self.add_inst_to_cursym(x64::Instruction::movmem_toreg64(
                    Reg64::RAX,
                    0,
                    Reg64::RAX,
                ));
                self.add_inst_to_cursym(x64::Instruction::pushreg64(Reg64::RAX));
                self.gen_comment("end identifier expression");
            }
            res::ExpressionNodeKind::CALL(ident, args) => {
                self.gen_comment("start call expression");
                for arg in args.iter() {
                    self.gen_expr(arg, local_map);
                }

                let arg_number: usize = if args.is_empty() { 0 } else { args.len() - 1 };

                for i in 0..args.len() {
                    let arg_reg = Self::caller_reg64(arg_number - i);
                    self.add_inst_to_cursym(x64::Instruction::popreg64(arg_reg));
                }

                self.add_inst_to_cursym(x64::Instruction::call(res::IdentName::last_name(ident)));

                self.add_inst_to_cursym(x64::Instruction::pushreg64(Reg64::RAX));

                self.gen_comment("end call expression");
            }

            // unary-expression
            res::ExpressionNodeKind::NEG(value) => {
                self.gen_unary_expr("-", value, local_map);
            }

            // binary-expression
            res::ExpressionNodeKind::ADD(lop, rop) => {
                self.gen_binary_expr("+", lop, rop, local_map)
            }
            res::ExpressionNodeKind::SUB(lop, rop) => {
                self.gen_binary_expr("-", lop, rop, local_map)
            }
            res::ExpressionNodeKind::MUL(lop, rop) => {
                self.gen_binary_expr("*", lop, rop, local_map)
            }
            res::ExpressionNodeKind::DIV(lop, rop) => {
                self.gen_binary_expr("/", lop, rop, local_map)
            }
            res::ExpressionNodeKind::ASSIGN(lval, rval) => {
                self.gen_comment("start assign expression");

                // 1． 左右子ノードをコンパイル
                //     左辺値はアドレスを生成し，スタックに積んでおく．
                self.gen_left_value(lval, local_map);
                self.gen_expr(rval, local_map);

                // 2．演算に必要なオペランドをレジスタに取り出す
                self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RDI));
                self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RAX));

                // 3．代入 == メモリに格納
                self.add_inst_to_cursym(x64::Instruction::movreg_tomem64(
                    Reg64::RDI,
                    Reg64::RAX,
                    0,
                ));

                // 4. 代入式のため，スタックにRDIの値を積んでおく
                self.add_inst_to_cursym(x64::Instruction::pushreg64(Reg64::RDI));

                self.gen_comment("end assign expression");
            }
            res::ExpressionNodeKind::IF(condition, body) => {
                self.gen_comment("start if expression");

                self.gen_expr(condition, local_map);
                let fin_label = format!(".Lend{}", self.consume_label());

                // condition
                self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RAX));
                self.add_inst_to_cursym(x64::Instruction::cmpreg_andint64(0, Reg64::RAX));
                self.add_inst_to_cursym(x64::Instruction::jump_equal_label(fin_label.clone()));

                for st in body.iter() {
                    self.gen_insts_from_statement(st, local_map);
                }

                self.add_inst_to_cursym(x64::Instruction::label(fin_label));

                self.gen_comment("end if expression");
            }
            res::ExpressionNodeKind::IFELSE(condition, body, alter) => {
                self.gen_comment("start if-else expression");

                self.gen_expr(condition, local_map);
                let label_num = self.consume_label();
                let else_label = format!(".Lelse{}", label_num);
                let fin_label = format!(".Lend{}", label_num);

                // condition
                self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RAX));
                self.add_inst_to_cursym(x64::Instruction::cmpreg_andint64(0, Reg64::RAX));
                self.add_inst_to_cursym(x64::Instruction::jump_equal_label(else_label.clone()));

                for st in body.iter() {
                    self.gen_insts_from_statement(st, local_map);
                }

                self.add_inst_to_cursym(x64::Instruction::jump_label(fin_label.clone()));
                self.add_inst_to_cursym(x64::Instruction::label(else_label));

                for st in alter.iter() {
                    self.gen_insts_from_statement(st, local_map);
                }

                self.add_inst_to_cursym(x64::Instruction::label(fin_label));

                self.gen_comment("end if-else expression");
            }
            _ => panic!("not implemented {} in gen_expr()", ex),
        }
    }

    pub fn gen_unary_expr(
        &mut self,
        operator: &str,
        value: &res::ExpressionNode,
        local_map: &BTreeMap<String, res::PVariable>,
    ) {
        // 1． 子ノードをコンパイル
        self.gen_expr(value, local_map);

        // 2．演算に必要なオペランドをレジスタに取り出す
        self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RAX));

        // 3．各演算に対応する命令を生成する
        match operator {
            "-" => self.add_inst_to_cursym(x64::Instruction::negreg64(Reg64::RAX)),
            _ => panic!("unsupported operator -> {}", operator),
        }

        // 4．演算結果をスタックに格納
        self.add_inst_to_cursym(x64::Instruction::pushreg64(x64::Reg64::RAX));
    }

    fn gen_binary_expr(
        &mut self,
        operator: &str,
        lop: &res::ExpressionNode,
        rop: &res::ExpressionNode,
        local_map: &BTreeMap<String, res::PVariable>,
    ) {
        // 1． 左右子ノードをコンパイル
        self.gen_expr(lop, local_map);
        self.gen_expr(rop, local_map);

        // 2．演算に必要なオペランドをレジスタに取り出す
        self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RDI));
        self.add_inst_to_cursym(x64::Instruction::popreg64(Reg64::RAX));

        // 3．各演算に対応する命令を生成する
        match operator {
            "+" => {
                self.add_inst_to_cursym(x64::Instruction::addreg_toreg64(Reg64::RDI, Reg64::RAX))
            }
            "-" => {
                self.add_inst_to_cursym(x64::Instruction::subreg_toreg64(Reg64::RDI, Reg64::RAX))
            }
            "*" => {
                self.add_inst_to_cursym(x64::Instruction::imulreg_toreg64(Reg64::RDI, Reg64::RAX))
            }
            "/" => {
                self.add_inst_to_cursym(x64::Instruction::cltd());
                self.add_inst_to_cursym(x64::Instruction::idivreg64(Reg64::RDI))
            }
            _ => panic!("unsupported operator -> {}", operator),
        }

        // 4．演算結果をスタックに格納
        self.add_inst_to_cursym(x64::Instruction::pushreg64(x64::Reg64::RAX));
    }

    fn gen_function_prologue(&mut self, offset: usize) {
        // save rbp
        self.add_inst_to_cursym(x64::Instruction::pushreg64(x64::Reg64::RBP));
        self.add_inst_to_cursym(x64::Instruction::movreg_toreg64(
            x64::Reg64::RSP,
            x64::Reg64::RBP,
        ));

        // allocating memory area for auto-var
        if offset != 0 {
            self.add_inst_to_cursym(x64::Instruction::subreg_byuint64(
                (!7 & offset + 7) as u64,
                x64::Reg64::RSP,
            ));
        }
    }

    fn gen_function_epilogue(&mut self) {
        self.add_inst_to_cursym(x64::Instruction::movreg_toreg64(
            x64::Reg64::RBP,
            x64::Reg64::RSP,
        ));
        self.add_inst_to_cursym(x64::Instruction::popreg64(x64::Reg64::RBP));
        self.add_inst_to_cursym(x64::Instruction::ret());
    }

    fn gen_left_value(
        &mut self,
        lval: &res::ExpressionNode,
        local_map: &BTreeMap<String, res::PVariable>,
    ) {
        match &lval.kind {
            res::ExpressionNodeKind::IDENT(id_name) => {
                let name = res::IdentName::last_name(id_name);
                let cur_pvar = local_map.get(&name);

                if cur_pvar.is_none() {
                    panic!("{} is not defined", name);
                }

                self.add_inst_to_cursym(x64::Instruction::movreg_toreg64(Reg64::RBP, Reg64::RAX));
                self.add_inst_to_cursym(x64::Instruction::subreg_byuint64(
                    cur_pvar.unwrap().get_stack_offset() as u64,
                    Reg64::RAX,
                ));
                self.add_inst_to_cursym(x64::Instruction::pushreg64(Reg64::RAX));
            }
            _ => panic!("can't generate {} as lvalue", lval.kind),
        }
    }

    fn gen_comment(&mut self, contents: &str) {
        self.add_inst_to_cursym(x64::Instruction::comment(contents.to_string()));
    }

    fn new(file_path: String) -> Self {
        Self {
            sym_idx: 0,
            asm: x64::AssemblyFile::new(file_path),
            label: 1,
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
    fn add_inst_to_cursym(&mut self, inst: x64::Instruction) {
        self.asm.add_inst_to_sym(self.sym_idx, inst);
    }

    fn caller_reg64(idx: usize) -> Reg64 {
        let regs = vec![
            Reg64::RDI,
            Reg64::RSI,
            Reg64::RDX,
            Reg64::RCX,
            Reg64::R8,
            Reg64::R9,
        ];
        regs[idx].clone()
    }
}