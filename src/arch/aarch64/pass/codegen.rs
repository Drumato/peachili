use crate::arch::aarch64::ir as lir;
use crate::common::analyze_resource::frame_object::StackFrame;
use crate::common::analyze_resource::peachili_type::Type;
use crate::common::three_address_code as tac;
use std::collections::BTreeMap;

pub fn codegen_main(ir_module: tac::IRModule, stack_frame: StackFrame) -> lir::Module {
    let mut aarch64_module: lir::Module = Default::default();

    for tac_fn_id in ir_module.funcs.iter() {
        let tac_fn = ir_module.get_fn(tac_fn_id);
        let aarch64_fn = gen_aarch64_fn(tac_fn, &stack_frame);
        aarch64_module.push_function(aarch64_fn);
    }

    aarch64_module
}

fn gen_aarch64_fn(tac_fn: &tac::IRFunction, stack_frame: &StackFrame) -> lir::Function {
    let mut aarch64_fn = lir::Function::new(&tac_fn.name);
    aarch64_fn.push_block("entry");

    let mut generator = FunctionGenerator::new(aarch64_fn, stack_frame);

    // prologue
    generator.gen_function_prologue();

    // 引数定義があったらその分storeする
    generator.gen_arguments_to_stack(tac_fn);

    for code_id in tac_fn.codes.iter() {
        let code = tac_fn.get_code(*code_id);
        generator.gen_aarch64_inst(tac_fn, code);
    }

    generator.gen_function_epilogue();

    generator.f
}

struct FunctionGenerator<'a> {
    f: lir::Function,
    param_count: usize,
    virt_to_phys: BTreeMap<tac::Value, lir::Register>,
    frame: &'a StackFrame,
}

#[allow(dead_code)]
impl<'a> FunctionGenerator<'a> {
    fn gen_aarch64_inst(&mut self, tac_fn: &tac::IRFunction, code: tac::Code) {
        match code.kind {
            tac::CodeKind::LABEL { name } => {
                self.f.push_block(&name);
            }
            tac::CodeKind::RETURN { value } => {
                let value = tac_fn.get_value(value);
                self.gen_return_inst(value);
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
            _ => eprintln!("unimplemented {:?}", code.kind),
        }
    }

    fn gen_return_inst(&mut self, value: tac::Value) {
        let value = self.operand_from_value(value);
        self.moved_reg_to_reg(
            value,
            self.new_reg_operand(lir::Register::GPR { number: 0 }),
        );

        // epilogue
        self.gen_function_epilogue();

        self.add_inst_to_last_bb(lir::InstKind::RET);
    }

    fn gen_call_inst(&mut self, called_name: String, result_value: tac::Value) {
        self.add_inst_to_last_bb(lir::InstKind::BL { name: called_name });

        let result = self.operand_from_value(result_value);
        let returned_reg = self.new_reg_operand(lir::Register::GPR { number: 0 });
        self.moved_reg_to_reg(result, returned_reg);
    }

    fn operand_from_value(&mut self, v: tac::Value) -> lir::Operand {
        match v.kind {
            tac::ValueKind::TEMP { number } => self.gen_phys_reg(number, v.ty),
            tac::ValueKind::INTLITERAL { value } => {
                lir::Operand::new(lir::OperandKind::IMMEDIATE { value })
            }
            tac::ValueKind::ID { name } => {
                let id_offset = self.get_local_var_offset(&name);
                self.new_memory_operand(lir::Register::FP, id_offset)
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

        self.add_inst_to_last_bb(lir::InstKind::MOV {
            operand_size: lir::OperandSize::DWORD,
            src: param_value,
            dst: param_reg,
        });
    }

    /// (w0/x0 は返り値として用いるため)
    fn get_param_register(&mut self, param_idx: usize) -> lir::Operand {
        if 6 < param_idx + 1 {
            panic!("callee register exhausted");
        }

        self.new_reg_operand(lir::Register::GPR {
            number: param_idx,
        })
    }
    fn gen_phys_reg(&mut self, virt_num: usize, ty: Type) -> lir::Operand {
        let virt_reg = tac::Value::new_temp(virt_num, ty);
        if let Some(phys_reg) = self.virt_to_phys.get(&virt_reg) {
            return self.new_reg_operand(*phys_reg);
        }

        let reg = match virt_reg.get_virt_number() % lir::Register::AVAILABLES {
            reg_number @ 0..=9 => lir::Register::GPR {
                number: reg_number + 10,
            },
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
            reg_number @ 0..=9 => lir::Register::GPR {
                number: reg_number + 10,
            },
            _ => panic!("phys register exhausted"),
        };

        self.virt_to_phys.insert(v, reg);
        self.new_reg_operand(reg)
    }

    fn new_reg_operand(&self, reg: lir::Register) -> lir::Operand {
        lir::Operand::new(lir::OperandKind::REGISTER { reg })
    }

    fn new_memory_operand(&self, base: lir::Register, offset: isize) -> lir::Operand {
        lir::Operand::new(lir::OperandKind::MEMORY { base, offset })
    }

    fn gen_function_prologue(&mut self) {
        // 関数フレームの確保，フレーム/リンクレジスタの保存，その領域を無視するコードの挿入
        self.subd_reg_by_imm_inst(lir::Register::SP, self.get_function_frame_size());
        self.stored_reg_pair_to_memory(
            lir::Register::FP,
            lir::Register::LINK,
            self.new_memory_operand(lir::Register::SP, 16),
        );

        self.addd_reg_and_imm_to_reg(lir::Register::FP, lir::Register::SP, 16);
    }

    fn gen_function_epilogue(&mut self) {
        // フレーム/リンクレジスタの復帰，スタックポインタの復帰
        self.loadd_reg_pair_from_memory(
            lir::Register::FP,
            lir::Register::LINK,
            self.new_memory_operand(lir::Register::SP, 16),
        );

        self.add_inst_to_last_bb(lir::InstKind::ADD {
            operand_size: lir::OperandSize::DWORD,
            dst: self.new_reg_operand(lir::Register::SP),
            lop: self.new_reg_operand(lir::Register::SP),
            rop: self.get_function_frame_size(),
        });
    }

    fn add_inst_to_last_bb(&mut self, inst_kind: lir::InstKind) {
        self.f.add_inst_to_last_bb(lir::Instruction::new(inst_kind));
    }

    fn addd_reg_and_imm_to_reg(&mut self, dst: lir::Register, lop: lir::Register, rop: usize) {
        self.add_inst_to_last_bb(lir::InstKind::ADD {
            operand_size: lir::OperandSize::DWORD,
            dst: self.new_reg_operand(dst),
            lop: self.new_reg_operand(lop),
            rop: lir::Operand::new(lir::OperandKind::IMMEDIATE { value: rop as i64 }),
        });
    }

    fn stored(&mut self, dst: lir::Operand, src: lir::Operand) {
        self.add_inst_to_last_bb(lir::InstKind::STR {
            operand_size: lir::OperandSize::DWORD,
            dst,
            src,
        });
    }
    fn stored_reg_pair_to_memory(
        &mut self,
        reg1: lir::Register,
        reg2: lir::Register,
        dst: lir::Operand,
    ) {
        self.add_inst_to_last_bb(lir::InstKind::STP {
            operand_size: lir::OperandSize::DWORD,
            reg1,
            reg2,
            dst,
        });
    }
    fn loadd_reg_pair_from_memory(
        &mut self,
        reg1: lir::Register,
        reg2: lir::Register,
        src: lir::Operand,
    ) {
        self.add_inst_to_last_bb(lir::InstKind::LDP {
            operand_size: lir::OperandSize::DWORD,
            reg1,
            reg2,
            src,
        });
    }

    fn subd_reg_by_imm_inst(&mut self, dst: lir::Register, value: lir::Operand) {
        self.add_inst_to_last_bb(lir::InstKind::SUB {
            operand_size: lir::OperandSize::DWORD,
            lop: self.new_reg_operand(dst),
            rop: value,
            dst: self.new_reg_operand(dst),
        });
    }
    fn moved_reg_to_reg(&mut self, dst: lir::Operand, src: lir::Operand) {
        self.add_inst_to_last_bb(lir::InstKind::MOV {
            operand_size: lir::OperandSize::DWORD,
            src,
            dst,
        });
    }

    fn gen_arguments_to_stack(&mut self, tac_fn: &tac::IRFunction) {
        for (arg_idx, arg_name) in tac_fn.args.iter().enumerate() {
            let param_reg = self.get_param_register(arg_idx);
            let memory_op =
                self.new_memory_operand(lir::Register::FP, self.get_local_var_offset(arg_name));

            self.stored(memory_op, param_reg);
        }
    }

    fn get_local_var_offset(&self, var_name: &str) -> isize {
        self.frame
            .get(self.f.get_name())
            .unwrap()
            .get(var_name)
            .unwrap()
            .offset as isize
    }

    fn get_function_frame_size(&self) -> lir::Operand {
        let base_size = self
            .frame
            .get(self.f.get_name())
            .unwrap()
            .get(self.f.get_name())
            .unwrap()
            .offset as i64;

        // x29,x30を格納する領域を確保
        let size = base_size + 16;

        lir::Operand::new(lir::OperandKind::IMMEDIATE { value: size })
    }

    fn new(aarch64_fn: lir::Function, stack_frame: &'a StackFrame) -> Self {
        Self {
            f: aarch64_fn,
            param_count: 0,
            virt_to_phys: Default::default(),
            frame: stack_frame,
        }
    }
}
