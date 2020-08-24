use std::collections::BTreeMap;

use crate::arch::aarch64::ir as lir;
use crate::common::analyze_resource::frame_object::StackFrame;
use crate::common::three_address_code as tac;

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
    // generator.gen_arguments_to_stack(tac_fn);

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
    virtual_number: usize,
    frame: &'a StackFrame,
}

impl<'a> FunctionGenerator<'a> {
    /// IRタイプごとに命令を生成する
    fn gen_aarch64_inst(&mut self, tac_fn: &tac::IRFunction, code: tac::Code) {
        match code.kind {
            tac::CodeKind::PARAM { value } => self.gen_param_inst(tac_fn, value),
            tac::CodeKind::CALL { name, result } => self.gen_call_inst(tac_fn, name, result),
            tac::CodeKind::ASM { value } => {
                let asm_literal = tac_fn.get_value(value);
                self.gen_inst_to_last_bb(lir::InstKind::INLINEASM {
                    contents: asm_literal.copy_contents(),
                });
            },
            _ => eprintln!("unimplemented {:?} inst", code.kind),
        }
    }

    fn gen_call_inst(&mut self, tac_fn: &tac::IRFunction, callee: tac::ValueId, result: tac::ValueId) {
        let called_name = tac_fn.get_called_name(callee);
        let result_value = tac_fn.get_value(result);

        self.gen_inst_to_last_bb(lir::InstKind::BL { name: called_name });
        let result_op = self.operand_from_value(result_value);

        self.gen_inst_to_last_bb(lir::InstKind::MOV {
            operand_size: lir::OperandSize::DWORD,
            dst: result_op,
            src: lir::Operand::new_register(lir::Register::GPR { number: 0 }),
        });

        self.param_count = 0;
    }

    /// 引数のpushを実装する
    fn gen_param_inst(&mut self, tac_fn: &tac::IRFunction, value_id: tac::ValueId) {
        let value = tac_fn.get_value(value_id);
        let param_value = self.operand_from_value(value);
        let param_reg = self.get_param_register();
        self.gen_inst_to_last_bb(lir::InstKind::MOV {
            operand_size: lir::OperandSize::DWORD,
            dst: param_reg,
            src: param_value,
        });

        self.param_count += 1;
    }

    /// 関数プロローグを生成する．
    fn gen_function_prologue(&mut self) {
        let stack_size = self.get_stack_size_from_current_function();

        // 関数フレームの割付
        self.gen_inst_to_last_bb(lir::InstKind::SUB {
            operand_size: lir::OperandSize::DWORD,
            dst: lir::Operand::new_register(lir::Register::SP),
            lop: lir::Operand::new_register(lir::Register::SP),
            rop: lir::Operand::new_immediate(stack_size as i64),
        });

        // fp, lr の保存
        self.gen_inst_to_last_bb(lir::InstKind::STP {
            operand_size: lir::OperandSize::DWORD,
            reg1: lir::Register::FP,
            reg2: lir::Register::LINK,
            dst: lir::Operand::new_memory(lir::Register::SP, stack_size as isize - 16),
        });

        // フレームポインタの更新
        // これは，fp/lrの保存に用いた領域を無視する為に挿入される
        self.gen_inst_to_last_bb(lir::InstKind::ADD {
            operand_size: lir::OperandSize::DWORD,
            dst: lir::Operand::new_register(lir::Register::FP),
            lop: lir::Operand::new_register(lir::Register::FP),
            rop: lir::Operand::new_immediate(16),
        });
    }

    /// 関数エピローグの生成
    fn gen_function_epilogue(&mut self) {
        let stack_size = self.get_stack_size_from_current_function();
        // フレーム/リンクレジスタの復帰，スタックポインタの復帰
        self.gen_inst_to_last_bb(lir::InstKind::LDP {
            operand_size: lir::OperandSize::DWORD,
            reg1: lir::Register::FP,
            reg2: lir::Register::LINK,
            src: lir::Operand::new_memory(lir::Register::SP, stack_size as isize - 16),
        });

        self.gen_inst_to_last_bb(lir::InstKind::ADD {
            operand_size: lir::OperandSize::DWORD,
            dst: lir::Operand::new_register(lir::Register::SP),
            lop: lir::Operand::new_register(lir::Register::SP),
            rop: lir::Operand::new_immediate(stack_size as i64),
        });
    }

    /// 三番地コードをaarch64の命令オペランドに変換する
    fn operand_from_value(&mut self, v: tac::Value) -> lir::Operand {
        match v.kind {
            tac::ValueKind::TEMP{ number: _ } => self.gen_physical_reg_from_temp(v),
            tac::ValueKind::INTLITERAL { value: int_value } => {
                let dst_reg = self.gen_physical_reg();

                self.gen_inst_to_last_bb(lir::InstKind::MOV {
                    operand_size: lir::OperandSize::DWORD,
                    dst: dst_reg,
                    src: lir::Operand::new_immediate(int_value),
                });

                dst_reg
            }
            tac::ValueKind::BOOLEANLITERAL { truth } => {
                let int_value = if truth {
                    lir::Operand::new_immediate(1)
                } else {
                    lir::Operand::new_immediate(0)
                };

                let dst_reg = self.gen_physical_reg();

                self.gen_inst_to_last_bb(lir::InstKind::MOV {
                    operand_size: lir::OperandSize::DWORD,
                    dst: dst_reg,
                    src: int_value,
                });

                dst_reg
            }
            _ => panic!("cannot generate from {:?}", v.kind),
        }
    }

    /// 引数渡すする際に用いるレジスタを取得する
    /// w0/x0 は返り値として用いるため + 1
    fn get_param_register(&mut self) -> lir::Operand {
        if 6 < self.param_count + 1 {
            panic!("callee register exhausted");
        }

        lir::Operand::new_register(lir::Register::GPR{number: self.param_count})
    }

    /// aarch64のために re-numbering しつつレジスタを生成する
    fn gen_physical_reg(&mut self) -> lir::Operand {
        let reg = match self.virtual_number % lir::Register::AVAILABLES {
            reg_number @ 0..=9 => lir::Register::GPR {
                number: reg_number + 10,
            },
            _ => panic!("phys register exhausted"),
        };

        self.virtual_number += 1;
        lir::Operand::new_register(reg)
    }

    fn gen_physical_reg_from_temp(&mut self, v: tac::Value) -> lir::Operand {
        let reg = match v.get_virt_number() % lir::Register::AVAILABLES {
            reg_number @ 0..=9 => lir::Register::GPR {
                number: reg_number + 10,
            },
            _ => panic!("phys register exhausted"),
        };

        lir::Operand::new_register(reg)
    }


    /// 関数フレームのサイズ取得
    /// スタックサイズは，fp/lrの保存のために増しておく
    fn get_stack_size_from_current_function(&self) -> usize {
        let fn_name = self.f.get_name();
        self.frame
            .get(fn_name)
            .unwrap()
            .get(fn_name)
            .unwrap()
            .offset
            + 16
    }
    fn gen_inst_to_last_bb(&mut self, ik: lir::InstKind) {
        self.f.add_inst_to_last_bb(lir::Instruction::new(ik));
    }
    fn new(aarch64_fn: lir::Function, stack_frame: &'a StackFrame) -> Self {
        Self {
            f: aarch64_fn,
            param_count: 0,
            virtual_number: 0,
            frame: stack_frame,
        }
    }
}
