use crate::common::arch::x64;
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
    }
    generator.give_assembly()
}

struct Generator {
    asm: x64::AssemblyFile,
    sym_idx: usize,
}

impl Generator {
    fn gen_symbol_from_func(&mut self, func: res::PFunction) {
        let symbol_name = func.copy_func_name();
        let this_sym = x64::Symbol::new(symbol_name);
        self.add_symbol(this_sym);

        let stack_offset = func.get_stack_offset();
        self.gen_function_prologue(stack_offset);

        for st in func.get_statements() {
            self.gen_insts_from_statement(st);
        }

        self.gen_function_epilogue();
    }

    fn gen_insts_from_statement(&mut self, st: &res::StatementNode) {
        match &st.kind {
            res::StatementNodeKind::RETURN(expr) => self.gen_return_statement(expr),
            res::StatementNodeKind::IFRET(expr) => self.gen_ifret_statement(expr),
            res::StatementNodeKind::EXPR(expr) => self.gen_expression_statement(expr),
            res::StatementNodeKind::VARDECL => (),
            res::StatementNodeKind::COUNTUP(_id, _start, _end, _body) => {
                panic!("not implement code-generating in countup-statement")
            }
        }
    }

    fn gen_return_statement(&mut self, expr: &res::ExpressionNode) {
        self.gen_comment("start return statement");

        self.gen_expr(expr);
        self.add_inst_to_cursym(x64::Instruction::popreg64(x64::Reg64::RAX));

        self.gen_comment("end return statement");
    }

    fn gen_ifret_statement(&mut self, expr: &res::ExpressionNode) {
        self.gen_comment("start ifret statement");

        self.gen_expr(expr);

        self.gen_comment("end ifret statement");
    }

    fn gen_expression_statement(&mut self, expr: &res::ExpressionNode) {
        self.gen_comment("start expression statement");

        self.gen_expr(expr);

        self.gen_comment("end expression statement");
    }

    fn gen_expr(&mut self, ex: &res::ExpressionNode) {
        match ex.kind {
            res::ExpressionNodeKind::INTEGER(v) => {
                self.add_inst_to_cursym(x64::Instruction::pushint64(v));
            }
            _ => panic!("not implemented {} in gen_expr()", ex),
        }
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

    fn gen_comment(&mut self, contents: &str) {
        self.add_inst_to_cursym(x64::Instruction::comment(contents.to_string()));
    }

    fn new(file_path: String) -> Self {
        Self {
            sym_idx: 0,
            asm: x64::AssemblyFile::new(file_path),
        }
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
}
