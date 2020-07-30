use crate::arch::x64::ir as lir;
use crate::common::analyze_resource::frame_object::StackFrame;
use crate::common::three_address_code as tac;
use std::collections::BTreeMap;
use crate::common::analyze_resource::peachili_type::Type;

pub fn codegen_main(ir_module: tac::IRModule, stack_frame: StackFrame) -> lir::Module {
    let mut x64_module: lir::Module = Default::default();

    for tac_fn_id in ir_module.funcs.iter() {
        let tac_fn = ir_module.get_fn(tac_fn_id);
        let x64_fn = gen_x64_fn(tac_fn, &stack_frame);
        x64_module.push_function(x64_fn);
    }

    x64_module
}

fn gen_x64_fn(tac_fn: &tac::IRFunction, stack_frame: &StackFrame) -> lir::Function {
    let mut x64_fn = lir::Function::new(&tac_fn.name);
    x64_fn.push_block("entry");

    let mut generator = FunctionGenerator::new(x64_fn, stack_frame);

    // prologue
    generator.gen_function_prologue();

    // 引数定義があったらその分storeする
    generator.gen_arguments_to_stack(tac_fn);

    for code_id in tac_fn.codes.iter() {
        let code = tac_fn.get_code(*code_id);
        generator.gen_x64_inst(tac_fn, code);
    }

    generator.f
}

struct FunctionGenerator<'a> {
    f: lir::Function,
    param_count: usize,
    virt_to_phys: BTreeMap<tac::Value, lir::Register>,
    frame: &'a StackFrame,
}

impl<'a> FunctionGenerator<'a> {
    fn gen_x64_inst(&mut self, tac_fn: &tac::IRFunction, code: tac::Code) {
        match code.kind {
            tac::CodeKind::LABEL { name } => {
                self.f.push_block(&name);
            }
            tac::CodeKind::RETURN { value } => {
                let value = tac_fn.get_value(value);
                self.gen_return_inst(value);
            }
            tac::CodeKind::ADD { lop, rop, result } => {
                let lop_value = tac_fn.get_value(lop);
                let rop_value = tac_fn.get_value(rop);
                let result = tac_fn.get_value(result);
                self.gen_add_inst(lop_value, rop_value, result);
            }
            tac::CodeKind::SUB { lop, rop, result } => {
                let lop_value = tac_fn.get_value(lop);
                let rop_value = tac_fn.get_value(rop);
                let result = tac_fn.get_value(result);
                self.gen_sub_inst(lop_value, rop_value, result);
            }
            tac::CodeKind::MUL { lop, rop, result } => {
                let lop_value = tac_fn.get_value(lop);
                let rop_value = tac_fn.get_value(rop);
                let result = tac_fn.get_value(result);
                self.gen_mul_inst(lop_value, rop_value, result);
            }
            tac::CodeKind::DIV { lop, rop, result } => {
                let lop_value = tac_fn.get_value(lop);
                let rop_value = tac_fn.get_value(rop);
                let result = tac_fn.get_value(result);
                self.gen_div_inst(lop_value, rop_value, result);
            }
            tac::CodeKind::ASSIGN { value, result } => {
                let value = tac_fn.get_value(value);
                let value_op = self.operand_from_value(value);
                let result = tac_fn.get_value(result);
                let result = self.operand_from_value(result);

                self.add_inst_to_last_bb(lir::InstKind::MOV {
                    operand_size: lir::OperandSize::QWORD,
                    src: value_op,
                    dst: result,
                });
            }
            tac::CodeKind::NEG { value, result } => {
                let value = tac_fn.get_value(value);
                let result = tac_fn.get_value(result);
                self.gen_neg_inst(value, result);
            }
            tac::CodeKind::ADDRESSOF { value, result } => {
                let value = tac_fn.get_value(value);
                let result = tac_fn.get_value(result);
                self.gen_address_inst(value, result);
            }
            tac::CodeKind::DEREFERENCE { value, result } => {
                let value = tac_fn.get_value(value);
                let result = tac_fn.get_value(result);
                self.gen_deref_inst(value, result);
            }

            tac::CodeKind::ASM { value } => {
                let asm_value = tac_fn.get_value(value);
                self.add_inst_to_last_bb(lir::InstKind::INLINEASM {
                    contents: asm_value.copy_contents(),
                });
            }

            tac::CodeKind::PARAM { value } => {
                let param_value = tac_fn.get_value(value);
                self.gen_param_inst(param_value);

                self.param_count += 1;
            }

            tac::CodeKind::CALL { name, result } => {
                let called_name = tac_fn.get_called_name(name);
                let result_value = tac_fn.get_value(result);

                self.gen_call_inst(called_name, result_value);

                self.param_count = 0;
            }

            tac::CodeKind::JUMP { label } => {
                self.add_inst_to_last_bb(lir::InstKind::JMP {
                    label: format!("{}_{}", self.f.get_name(), label),
                });
            }
            tac::CodeKind::STORE { value, result } => {
                let value = tac_fn.get_value(value);
                let value_op = self.operand_from_value(value);
                let result = tac_fn.get_value(result);
                let result_op = self.operand_from_value(result);

                self.storeq(
                    value_op,
                    self.new_memory_operand(result_op.get_reg(), 0),
                );
            }
            tac::CodeKind::JUMPIFFALSE { label, cond_result } => {
                let value = tac_fn.get_value(cond_result);
                let value_op = self.operand_from_value(value);

                self.add_inst_to_last_bb(lir::InstKind::CMP {
                    operand_size: lir::OperandSize::QWORD,
                    src: lir::Operand::new(lir::OperandKind::IMMEDIATE { value: 0 }),
                    dst: value_op,
                });
                self.add_inst_to_last_bb(lir::InstKind::JE {
                    label: format!("{}_{}", self.f.get_name(), label),
                });
            }

            tac::CodeKind::ALLOC { temp: _ } => {}
            _ => eprintln!("unimplemented {:?}", code.kind),
        }
    }

    fn gen_return_inst(&mut self, value: tac::Value) {
        let value = self.operand_from_value(value);
        self.moveq_reg_to_reg_inst(value, self.new_reg_operand(lir::Register::RAX));

        // epilogue
        self.gen_function_epilogue();

        self.add_inst_to_last_bb(lir::InstKind::RET);
    }

    fn gen_call_inst(&mut self, called_name: String, result_value: tac::Value) {
        self.add_inst_to_last_bb(lir::InstKind::CALL { name: called_name });

        let result = self.operand_from_value(result_value);
        let returned_reg = self.new_reg_operand(lir::Register::RAX);
        self.moveq_reg_to_reg_inst(returned_reg, result);
    }

    fn gen_add_inst(&mut self, lop: tac::Value, rop: tac::Value, result: tac::Value) {
        let result_reg = self.gen_phys_reg_from(result);
        let lop = self.operand_from_value(lop);
        let rop = self.operand_from_value(rop);

        match lop.get_kind() {
            // resultにmoveしてからplus
            lir::OperandKind::IMMEDIATE { value: _ } => {
                self.moveq_reg_to_reg_inst(lop, result_reg);
                self.addq_reg_and_reg(rop, result_reg);
            }
            // lopにplusしてresultに格納
            lir::OperandKind::REGISTER { reg: _ } => {
                self.addq_reg_and_reg(rop, lop);
                self.moveq_reg_to_reg_inst(lop, result_reg);
            }
            // resultにmoveしてからplus
            lir::OperandKind::MEMORY { base: _, offset: _ } => {
                self.moveq_reg_to_reg_inst(lop, result_reg);
                self.addq_reg_and_reg(rop, result_reg);
            }
        }
    }
    fn gen_sub_inst(&mut self, lop: tac::Value, rop: tac::Value, result: tac::Value) {
        let result_reg = self.gen_phys_reg_from(result);
        let lop = self.operand_from_value(lop);
        let rop = self.operand_from_value(rop);

        match lop.get_kind() {
            // resultにmoveしてからplus
            lir::OperandKind::IMMEDIATE { value: _ } => {
                self.moveq_reg_to_reg_inst(lop, result_reg);
                self.subq_reg_and_reg(rop, result_reg);
            }
            // lopにplusしてresultに格納
            lir::OperandKind::REGISTER { reg: _ } => {
                self.subq_reg_and_reg(rop, lop);
                self.moveq_reg_to_reg_inst(lop, result_reg);
            }
            // resultにmoveしてからplus
            lir::OperandKind::MEMORY { base: _, offset: _ } => {
                self.moveq_reg_to_reg_inst(lop, result_reg);
                self.subq_reg_and_reg(rop, result_reg);
            }
        }
    }
    fn gen_mul_inst(&mut self, lop: tac::Value, rop: tac::Value, result: tac::Value) {
        let result_reg = self.gen_phys_reg_from(result);
        let lop = self.operand_from_value(lop);
        let rop = self.operand_from_value(rop);

        match lop.get_kind() {
            // resultにmoveしてからplus
            lir::OperandKind::IMMEDIATE { value: _ } => {
                self.moveq_reg_to_reg_inst(lop, result_reg);
                self.imulq_reg_and_reg(rop, result_reg);
            }
            // lopにplusしてresultに格納
            lir::OperandKind::REGISTER { reg: _ } => {
                self.imulq_reg_and_reg(rop, lop);
                self.moveq_reg_to_reg_inst(lop, result_reg);
            }
            // resultにmoveしてからplus
            lir::OperandKind::MEMORY { base: _, offset: _ } => {
                self.moveq_reg_to_reg_inst(lop, result_reg);
                self.imulq_reg_and_reg(rop, result_reg);
            }
        }
    }

    fn gen_div_inst(&mut self, lop: tac::Value, rop: tac::Value, result: tac::Value) {
        let result_reg = self.gen_phys_reg_from(result);
        let lop = self.operand_from_value(lop);
        let rop = self.operand_from_value(rop);

        // とりあえずスタック使う
        // rdiを引数レジスタとして使っているかもしれないから保存
        self.pushq_reg_inst(lir::Register::RDI);
        self.moveq_reg_to_reg_inst(lop, self.new_reg_operand(lir::Register::RAX));
        self.moveq_reg_to_reg_inst(rop, self.new_reg_operand(lir::Register::RDI));
        self.add_inst_to_last_bb(lir::InstKind::CLTD);
        self.idivq_rax_by_reg(lir::Register::RDI);
        self.moveq_reg_to_reg_inst(self.new_reg_operand(lir::Register::RAX), result_reg);
        self.popq_reg_inst(lir::Register::RDI);
    }

    fn gen_neg_inst(&mut self, value: tac::Value, result: tac::Value) {
        let result_reg = self.gen_phys_reg_from(result);
        let value_op = self.operand_from_value(value);

        match value_op.get_kind() {
            // resultにmoveしてからplus
            lir::OperandKind::IMMEDIATE { value: _ } => {
                self.moveq_reg_to_reg_inst(value_op, result_reg);
                self.negq_reg(result_reg);
            }
            // lopにplusしてresultに格納
            lir::OperandKind::REGISTER { reg: _ } => {
                self.negq_reg(value_op);
                self.moveq_reg_to_reg_inst(value_op, result_reg);
            }
            // resultにmoveしてからplus
            lir::OperandKind::MEMORY { base: _, offset: _ } => {
                self.negq_reg(value_op);
                self.moveq_reg_to_reg_inst(value_op, result_reg);
            }
        }
    }
    fn gen_address_inst(&mut self, value: tac::Value, result: tac::Value) {
        let result_reg = self.gen_phys_reg_from(result);
        let value_op = self.operand_from_value(value);

        match value_op.get_kind() {
            lir::OperandKind::IMMEDIATE { value: _ } => unreachable!(),
            lir::OperandKind::REGISTER { reg: _ } => unreachable!(),
            lir::OperandKind::MEMORY { base: _, offset: _ } => {
                self.leaq_memory_to_reg(value_op, result_reg);
            }
        }
    }
    fn gen_deref_inst(&mut self, value: tac::Value, result: tac::Value) {
        let result_reg = self.gen_phys_reg_from(result);
        let value_op = self.operand_from_value(value);

        match value_op.get_kind() {
            lir::OperandKind::IMMEDIATE { value: _ } => unreachable!(),
            lir::OperandKind::REGISTER { reg: _ } => {
                self.storeq(value_op, result_reg);
                self.storeq(self.new_memory_operand(result_reg.get_reg(), 0), result_reg);
            },
            lir::OperandKind::MEMORY { base: _, offset: _ } => {
                self.storeq(value_op, result_reg);
                self.storeq(self.new_memory_operand(result_reg.get_reg(), 0), result_reg);
            }
        }
    }

    fn operand_from_value(&mut self, v: tac::Value) -> lir::Operand {
        match v.kind {
            tac::ValueKind::TEMP { number } => self.gen_phys_reg(number, v.ty),
            tac::ValueKind::INTLITERAL { value } => {
                lir::Operand::new(lir::OperandKind::IMMEDIATE { value })
            }
            tac::ValueKind::ID { name } => {
                let id_offset = self.get_local_var_offset(&name);
                self.new_memory_operand(lir::Register::RBP, id_offset)
            }
            tac::ValueKind::BOOLEANLITERAL { truth } => {
                if truth {
                    lir::Operand::new(lir::OperandKind::IMMEDIATE { value: 1 })
                } else {
                    lir::Operand::new(lir::OperandKind::IMMEDIATE { value: 0 })
                }
            }
            tac::ValueKind::UINTLITERAL { value } => {
                lir::Operand::new(lir::OperandKind::IMMEDIATE {
                    value: value as i64,
                })
            }
            _ => unreachable!(),
        }
    }

    fn gen_param_inst(&mut self, param_value: tac::Value) {
        let param_value = self.operand_from_value(param_value);

        let param_reg = self.get_param_register(self.param_count);
        match param_value.get_kind() {
            lir::OperandKind::IMMEDIATE { value: _ } => {
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
                        operand_size: lir::OperandSize::QWORD,
                        src: param_value,
                        dst: param_reg,
                    }));
            }
            lir::OperandKind::REGISTER { reg: _ } => {
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
                        operand_size: lir::OperandSize::QWORD,
                        src: param_value,
                        dst: param_reg,
                    }));
            }
            lir::OperandKind::MEMORY { base: _, offset: _ } => {
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
                        operand_size: lir::OperandSize::QWORD,
                        src: param_value,
                        dst: param_reg,
                    }));
            }
        }
    }

    fn get_param_register(&mut self, param_idx: usize) -> lir::Operand {
        let reg = match param_idx {
            0 => lir::Register::RDI,
            1 => lir::Register::RSI,
            2 => lir::Register::RDX,
            3 => lir::Register::RCX,
            4 => lir::Register::R8,
            5 => lir::Register::R9,
            _ => panic!("callee register exhausted"),
        };

        self.new_reg_operand(reg)
    }

    fn gen_phys_reg(&mut self, virt_num: usize, ty: Type) -> lir::Operand {
        let virt_reg = tac::Value::new_temp(virt_num, ty);
        if let Some(phys_reg) = self.virt_to_phys.get(&virt_reg) {
            return self.new_reg_operand(*phys_reg);
        }

        let reg = match virt_num % lir::Register::AVAILABLES {
            0 => lir::Register::R10,
            1 => lir::Register::R11,
            2 => lir::Register::R12,
            3 => lir::Register::R13,
            4 => lir::Register::R14,
            5 => lir::Register::R15,
            _ => panic!("phys register exhausted"),
        };

        self.virt_to_phys.insert(virt_reg, reg);

        self.new_reg_operand(reg)
    }

    fn gen_phys_reg_from(&mut self, v: tac::Value) -> lir::Operand {
        if let Some(phys_reg) = self.virt_to_phys.get(&v) {
            return self.new_reg_operand(*phys_reg);
        }

        let reg = match v.get_virt_number() % lir::Register::AVAILABLES {
            0 => lir::Register::R10,
            1 => lir::Register::R11,
            2 => lir::Register::R12,
            3 => lir::Register::R13,
            4 => lir::Register::R14,
            5 => lir::Register::R15,
            _ => panic!("phys register exhausted"),
        };

        self.virt_to_phys.insert(v, reg);

        self.new_reg_operand(reg)
    }

    fn new_reg_operand(&self, reg: lir::Register) -> lir::Operand {
        lir::Operand::new(lir::OperandKind::REGISTER { reg })
    }
    fn new_memory_operand(&self, base: lir::Register, offset: usize) -> lir::Operand {
        lir::Operand::new(lir::OperandKind::MEMORY { base, offset })
    }

    fn gen_function_prologue(&mut self) {
        self.pushq_reg_inst(lir::Register::RBP);
        self.moveq_reg_to_reg_inst(
            self.new_reg_operand(lir::Register::RSP),
            self.new_reg_operand(lir::Register::RBP),
        );
        self.subq_reg_by_imm_inst(self.get_function_frame_size(), lir::Register::RSP);
    }

    fn gen_function_epilogue(&mut self) {
        self.moveq_reg_to_reg_inst(
            self.new_reg_operand(lir::Register::RBP),
            self.new_reg_operand(lir::Register::RSP),
        );
        self.popq_reg_inst(lir::Register::RBP);
    }

    fn pushq_reg_inst(&mut self, reg: lir::Register) {
        self.add_inst_to_last_bb(lir::InstKind::PUSH {
            operand_size: lir::OperandSize::QWORD,
            value: self.new_reg_operand(reg),
        });
    }
    fn popq_reg_inst(&mut self, reg: lir::Register) {
        self.add_inst_to_last_bb(lir::InstKind::POP {
            operand_size: lir::OperandSize::QWORD,
            value: self.new_reg_operand(reg),
        });
    }
    fn negq_reg(&mut self, reg: lir::Operand) {
        self.add_inst_to_last_bb(lir::InstKind::NEG {
            operand_size: lir::OperandSize::QWORD,
            value: reg,
        });
    }

    fn subq_reg_by_imm_inst(&mut self, value: lir::Operand, dst: lir::Register) {
        self.add_inst_to_last_bb(lir::InstKind::SUB {
            operand_size: lir::OperandSize::QWORD,
            src: value,
            dst: self.new_reg_operand(dst),
        });
    }

    fn moveq_reg_to_reg_inst(&mut self, src: lir::Operand, dst: lir::Operand) {
        self.add_inst_to_last_bb(lir::InstKind::MOV {
            operand_size: lir::OperandSize::QWORD,
            src,
            dst,
        });
    }
    fn addq_reg_and_reg(&mut self, src: lir::Operand, dst: lir::Operand) {
        self.add_inst_to_last_bb(lir::InstKind::ADD {
            operand_size: lir::OperandSize::QWORD,
            src,
            dst,
        });
    }
    fn subq_reg_and_reg(&mut self, src: lir::Operand, dst: lir::Operand) {
        self.add_inst_to_last_bb(lir::InstKind::SUB {
            operand_size: lir::OperandSize::QWORD,
            src,
            dst,
        });
    }
    fn imulq_reg_and_reg(&mut self, src: lir::Operand, dst: lir::Operand) {
        self.add_inst_to_last_bb(lir::InstKind::IMUL {
            operand_size: lir::OperandSize::QWORD,
            src,
            dst,
        });
    }
    fn idivq_rax_by_reg(&mut self, reg: lir::Register) {
        self.add_inst_to_last_bb(lir::InstKind::IDIV {
            operand_size: lir::OperandSize::QWORD,
            value: self.new_reg_operand(reg),
        });
    }
    fn leaq_memory_to_reg(&mut self, src: lir::Operand, dst: lir::Operand) {
        self.add_inst_to_last_bb(lir::InstKind::LEA {
            operand_size: lir::OperandSize::QWORD,
            src,
            dst,
        });
    }

    fn add_inst_to_last_bb(&mut self, inst_kind: lir::InstKind) {
        self.f.add_inst_to_last_bb(lir::Instruction::new(inst_kind));
    }

    fn storeq(&mut self, src: lir::Operand, dst: lir::Operand) {
        self.add_inst_to_last_bb(lir::InstKind::MOV {
            operand_size: lir::OperandSize::QWORD,
            dst,
            src,
        });
    }

    fn gen_arguments_to_stack(&mut self, tac_fn: &tac::IRFunction) {
        for (arg_idx, arg_name) in tac_fn.args.iter().enumerate() {
            let param_reg = self.get_param_register(arg_idx);
            let memory_op =
                self.new_memory_operand(lir::Register::RBP, self.get_local_var_offset(arg_name));

            self.storeq(param_reg, memory_op);
        }
    }

    fn get_local_var_offset(&self, var_name: &str) -> usize {
        self.frame
            .get(self.f.get_name())
            .unwrap()
            .get(var_name)
            .unwrap()
            .offset
    }

    fn get_function_frame_size(&self) -> lir::Operand {
        lir::Operand::new(lir::OperandKind::IMMEDIATE {
            value: self
                .frame
                .get(self.f.get_name())
                .unwrap()
                .get(self.f.get_name())
                .unwrap()
                .offset as i64,
        })
    }

    fn new(x64_fn: lir::Function, stack_frame: &'a StackFrame) -> Self {
        Self {
            f: x64_fn,
            param_count: 0,
            virt_to_phys: Default::default(),
            frame: stack_frame,
        }
    }
}
