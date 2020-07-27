use crate::arch::x64::ir as lir;
use crate::common::analyze_resource::frame_object::StackFrame;
use crate::common::three_address_code as tac;
use std::collections::BTreeMap;

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

    let mut generator = FunctionGenerator {
        f: x64_fn,
        param_count: 0,
        virt_to_phys: BTreeMap::new(),
        frame: stack_frame,
    };

    // prologue
    generator
        .f
        .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::PUSH {
            operand_size: lir::OperandSize::QWORD,
            value: lir::Operand::new(lir::OperandKind::REGISTER {
                reg: lir::Register::RBP,
            }),
        }));
    generator
        .f
        .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
            operand_size: lir::OperandSize::QWORD,
            src: lir::Operand::new(lir::OperandKind::REGISTER {
                reg: lir::Register::RSP,
            }),
            dst: lir::Operand::new(lir::OperandKind::REGISTER {
                reg: lir::Register::RBP,
            }),
        }));
    generator
        .f
        .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::SUB {
            operand_size: lir::OperandSize::QWORD,
            src: lir::Operand::new(lir::OperandKind::IMMEDIATE {
                value: generator
                    .frame
                    .get(generator.f.get_name())
                    .unwrap()
                    .get(generator.f.get_name())
                    .unwrap()
                    .offset as i64,
            }),
            dst: lir::Operand::new(lir::OperandKind::REGISTER {
                reg: lir::Register::RSP,
            }),
        }));

    // 引数定義があったらその分moveする
    for (arg_idx, arg_name) in tac_fn.args.iter().enumerate() {
        let param_reg = generator.get_param_register(arg_idx);
        generator
            .f
            .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
                operand_size: lir::OperandSize::QWORD,
                dst: lir::Operand::new(lir::OperandKind::MEMORY {
                    base: lir::Register::RBP,
                    offset: generator
                        .frame
                        .get(generator.f.get_name())
                        .unwrap()
                        .get(arg_name)
                        .unwrap()
                        .offset,
                }),
                src: param_reg,
            }));
    }

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
                let value = self.operand_from_value(value);
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
                        operand_size: lir::OperandSize::QWORD,
                        src: value,
                        dst: lir::Operand::new(lir::OperandKind::REGISTER {
                            reg: lir::Register::RAX,
                        }),
                    }));

                // epilogue
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
                        operand_size: lir::OperandSize::QWORD,
                        dst: lir::Operand::new(lir::OperandKind::REGISTER {
                            reg: lir::Register::RSP,
                        }),
                        src: lir::Operand::new(lir::OperandKind::REGISTER {
                            reg: lir::Register::RBP,
                        }),
                    }));
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::POP {
                        operand_size: lir::OperandSize::QWORD,
                        value: lir::Operand::new(lir::OperandKind::REGISTER {
                            reg: lir::Register::RBP,
                        }),
                    }));

                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::RET));
            }
            tac::CodeKind::ADD { lop, rop, result } => {
                let lop_value = tac_fn.get_value(lop);
                let rop_value = tac_fn.get_value(rop);
                let result = tac_fn.get_value(result);
                self.gen_add_inst(lop_value, rop_value, result);
            }

            tac::CodeKind::ASM { value } => {
                let asm_value = tac_fn.get_value(value);
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::INLINEASM {
                        contents: asm_value.copy_contents(),
                    }));
            }

            tac::CodeKind::PARAM { value } => {
                let param_value = tac_fn.get_value(value);
                self.gen_param_inst(param_value);

                self.param_count += 1;
            }

            tac::CodeKind::CALL { name, result } => {
                let called_name = tac_fn.get_called_name(name);
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::CALL {
                        name: called_name,
                    }));
                let result_value = tac_fn.get_value(result);
                let result = self.operand_from_value(result_value);
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
                        operand_size: lir::OperandSize::QWORD,
                        dst: result,
                        src: lir::Operand::new(lir::OperandKind::REGISTER {
                            reg: lir::Register::RAX,
                        }),
                    }));

                self.param_count = 0;
            }

            _ => eprintln!("unimplemented {:?}", code.kind),
        }
    }

    fn gen_add_inst(&mut self, lop: tac::Value, rop: tac::Value, result: tac::Value) {
        let result_reg = self.gen_phys_reg_from(result);
        let lop = self.operand_from_value(lop);
        let rop = self.operand_from_value(rop);

        match lop.get_kind() {
            // resultにmoveしてからplus
            lir::OperandKind::IMMEDIATE { value: _ } => {
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
                        operand_size: lir::OperandSize::QWORD,
                        src: lop,
                        dst: result_reg,
                    }));

                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::ADD {
                        operand_size: lir::OperandSize::QWORD,
                        src: rop,
                        dst: result_reg,
                    }));
            }
            // lopにplusしてresultに格納
            lir::OperandKind::REGISTER { reg: _ } => {
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::ADD {
                        operand_size: lir::OperandSize::QWORD,
                        src: rop,
                        dst: lop,
                    }));
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
                        operand_size: lir::OperandSize::QWORD,
                        src: lop,
                        dst: result_reg,
                    }));
            }
            // resultにmoveしてからplus
            lir::OperandKind::MEMORY { base: _, offset: _ } => {
                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::MOV {
                        operand_size: lir::OperandSize::QWORD,
                        src: lop,
                        dst: result_reg,
                    }));

                self.f
                    .add_inst_to_last_bb(lir::Instruction::new(lir::InstKind::ADD {
                        operand_size: lir::OperandSize::QWORD,
                        src: rop,
                        dst: result_reg,
                    }));
            }
        }
    }

    fn operand_from_value(&mut self, v: tac::Value) -> lir::Operand {
        match v.kind {
            tac::ValueKind::TEMP { number } => self.gen_phys_reg(number),
            tac::ValueKind::INTLITERAL { value } => {
                lir::Operand::new(lir::OperandKind::IMMEDIATE { value })
            }
            tac::ValueKind::ID { name } => lir::Operand::new(lir::OperandKind::MEMORY {
                base: lir::Register::RBP,
                offset: self
                    .frame
                    .get(self.f.get_name())
                    .unwrap()
                    .get(&name)
                    .unwrap()
                    .offset,
            }),
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

        lir::Operand::new(lir::OperandKind::REGISTER { reg })
    }

    fn gen_phys_reg(&mut self, virt_num: usize) -> lir::Operand {
        let virt_reg = tac::Value {
            kind: tac::ValueKind::TEMP { number: virt_num },
        };
        if let Some(phys_reg) = self.virt_to_phys.get(&virt_reg) {
            return lir::Operand::new(lir::OperandKind::REGISTER { reg: *phys_reg });
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

        lir::Operand::new(lir::OperandKind::REGISTER { reg })
    }

    fn gen_phys_reg_from(&mut self, v: tac::Value) -> lir::Operand {
        if let Some(phys_reg) = self.virt_to_phys.get(&v) {
            return lir::Operand::new(lir::OperandKind::REGISTER { reg: *phys_reg });
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

        lir::Operand::new(lir::OperandKind::REGISTER { reg })
    }
}
