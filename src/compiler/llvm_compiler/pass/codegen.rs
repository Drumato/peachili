use std::collections::BTreeMap;

use llvm_scratch::core::function::ParameterSet;
use llvm_scratch::core::{
    basic_block::BasicBlockId,
    function::{FunctionId, Parameter, ReturnType},
    instruction::{InstKind as LLVMInstKind, Instruction as LLVMInst},
    intrinsics::Intrinsic,
    llvm_string::LLVMString,
    llvm_type::LLVMType,
    llvm_value::LLVMValue,
    module::Module as LLVMModule,
    target_datalayout, target_triple,
};

use crate::common::{module, option};
use crate::compiler::general::resource as res;

pub fn codegen(
    build_option: &option::BuildOption,
    ast_root: res::ASTRoot,
    module_allocator: &module::ModuleAllocator,
) -> llvm_scratch::core::module::Module {
    let mut generator = Generator {
        m: create_module(build_option),
        label: 0,
        insert_bb: None,
    };

    generator.m.add_intrinsic(Intrinsic::DONOTHING);

    let functions = ast_root.get_functions();

    for (_func_name_id, func) in functions.iter() {
        let const_pool = module_allocator
            .get_module_ref(&func.module_id)
            .unwrap()
            .get_const_pool_ref();

        generator.gen_fn(func, &const_pool, functions);
    }

    generator.give_module()
}

struct Generator {
    m: LLVMModule,
    label: usize,

    insert_bb: Option<BasicBlockId>,
}

#[allow(dead_code)]
impl Generator {
    fn gen_fn(
        &mut self,
        func: &res::PFunction,
        const_pool: &res::ConstAllocator,
        func_map: &BTreeMap<res::PStringId, res::PFunction>,
    ) {
        let local_map = func.get_locals();
        let llvm_func_id = self.gen_func_prototype(func, const_pool, local_map);

        for st in func.get_statements() {
            self.gen_statement(llvm_func_id, st, func, func_map, const_pool);
        }

        if self.entry_bb_is_empty(llvm_func_id) {
            let donothing = Intrinsic::DONOTHING;
            let donothing_call = LLVMInst::new(
                LLVMInstKind::CALL(
                    None,
                    None,
                    Ok(ReturnType::new(donothing.return_type(), None)),
                    LLVMString::from("llvm.donothing"),
                    ParameterSet::new(Vec::new()),
                ),
                None,
            );
            self.insert_inst(llvm_func_id, donothing_call);
            self.insert_inst(llvm_func_id, LLVMInst::new(LLVMInstKind::RETVOID, None));
        }
    }

    fn gen_statement(
        &mut self,
        llvm_func_id: FunctionId,
        st: &res::StatementNode,
        func: &res::PFunction,
        func_map: &BTreeMap<res::PStringId, res::PFunction>,
        const_pool: &res::ConstAllocator,
    ) {
        match &st.kind {
            res::StatementNodeKind::RETURN(_return_expr) => {}
            res::StatementNodeKind::IFRET(_return_expr) => {}
            res::StatementNodeKind::EXPR(expr) => {
                self.gen_expression(llvm_func_id, expr, func, func_map, const_pool);
            }
            res::StatementNodeKind::VARDECL => {}
            res::StatementNodeKind::COUNTUP(_ident, _start_expr, _end_expr, _body) => {}
            res::StatementNodeKind::ASM(_asm_literals) => {}
            res::StatementNodeKind::VARINIT(_ident, _expr) => {}
        }
    }

    fn gen_expression(
        &mut self,
        llvm_func_id: FunctionId,
        ex: &res::ExpressionNode,
        func: &res::PFunction,
        func_map: &BTreeMap<res::PStringId, res::PFunction>,
        const_pool: &res::ConstAllocator,
    ) -> (LLVMValue, LLVMType) {
        match &ex.kind {
            res::ExpressionNodeKind::INTEGER(v) => {
                (LLVMValue::new_integer(*v as i128), LLVMType::new_int(64))
            }
            res::ExpressionNodeKind::UNSIGNEDINTEGER(v) => (
                LLVMValue::new_unsigned_int(*v as u128),
                LLVMType::new_int(64),
            ),

            res::ExpressionNodeKind::CALL(callee_name, args) => {
                let callee_name_id = res::IdentName::last_name(&callee_name);
                let called_func = func_map.get(&callee_name_id).unwrap();
                let callee_function_type = called_func.get_return_type();
                let callee_function_type = self.get_llvm_type_from_ptype(callee_function_type);
                let callee_func_args = called_func.get_args();

                let _func_args = func.get_args();
                let mut params = Vec::new();

                for (idx, arg) in args.iter().enumerate() {
                    let arg_name_id = callee_func_args[idx];
                    let arg_name = const_pool.get(arg_name_id).unwrap();

                    let (_param_value, param_ty) =
                        self.gen_expression(llvm_func_id, arg, func, func_map, const_pool);

                    params.push(Parameter::new(
                        LLVMString::from(arg_name.copy_value()),
                        None,
                        param_ty,
                    ));
                }

                let param_set = ParameterSet::new(params);
                let call_result = self.consume_cur_register();

                let callee_name = const_pool.get(callee_name_id).unwrap();

                let call_inst = LLVMInst::new(
                    LLVMInstKind::CALL(
                        None,
                        None,
                        Ok(ReturnType::new(callee_function_type.clone(), None)),
                        LLVMString::from(callee_name.copy_value()),
                        param_set,
                    ),
                    Some(call_result.clone()),
                );
                self.insert_inst(llvm_func_id, call_inst);

                (call_result, callee_function_type)
            }
            _ => panic!("unimplemented gen_expression -> {}", ex),
        }
    }

    fn gen_func_prototype(
        &mut self,
        func: &res::PFunction,
        const_pool: &res::ConstAllocator,
        local_map: &BTreeMap<Vec<res::PStringId>, res::PVariable>,
    ) -> FunctionId {
        // 関数名, 返り値の型からアウトラインを作る
        let func_name = const_pool
            .get(func.get_func_name_id())
            .unwrap()
            .copy_value();
        let func_ret_type = self.get_llvm_type_from_ptype(func.get_return_type());
        let fn_proto_id = self
            .m
            .new_function(&func_name, ReturnType::new(func_ret_type, None));

        self.insert_bb = Some(self.m.get_function_ref(fn_proto_id).get_entry_bb());

        // 引数があれば，allocとstoreを引数分発行
        for (_arg_i, arg_name_id) in func.get_args().iter().enumerate() {
            let arg_name = LLVMString::from(const_pool.get(*arg_name_id).unwrap().copy_value());
            let arg_var = local_map.get(vec![*arg_name_id].as_slice()).unwrap();
            let arg_type = self.get_llvm_type_from_ptype(arg_var.get_type());
            let param = Parameter::new(arg_name, None, arg_type);

            self.insert_parameter(fn_proto_id, param);

            let alloc_inst = self.gen_arg_allocate(arg_var);
            self.insert_inst(fn_proto_id, alloc_inst);
        }

        fn_proto_id
    }

    fn gen_arg_allocate(&mut self, arg_var: &res::PVariable) -> LLVMInst {
        let llvm_ty = self.get_llvm_type_from_ptype(arg_var.get_type());
        let alignment = 8;

        let arg_reg = self.consume_cur_register();
        LLVMInst::new(
            LLVMInstKind::ALLOCA(llvm_ty, None, Some(alignment), None),
            Some(arg_reg),
        )
    }

    fn get_llvm_type_from_ptype(&mut self, ty: &res::PType) -> LLVMType {
        match &ty.kind {
            res::PTypeKind::INT64 => LLVMType::new_int(64),
            res::PTypeKind::UINT64 => LLVMType::new_uint(64),
            res::PTypeKind::CONSTSTR => LLVMType::new_pointer(LLVMType::new_int(8)),
            res::PTypeKind::NORETURN => LLVMType::new_void(),
            res::PTypeKind::BOOLEAN => LLVMType::new_int(1),
            res::PTypeKind::UNRESOLVED(_) => {
                panic!("unimplemented getting llvm-type from unresolved")
            }
            res::PTypeKind::POINTER(inner, _ref_local) => {
                LLVMType::new_pointer(self.get_llvm_type_from_ptype(inner))
            }
            res::PTypeKind::INVALID => panic!("unimplemented getting llvm-type from invalid"),
        }
    }

    fn insert_parameter(&mut self, func_id: FunctionId, param: Parameter) {
        let func = self.m.get_function_ref_as_mut(func_id);
        func.new_argument(param);
    }

    fn insert_inst(&mut self, func_id: FunctionId, inst: LLVMInst) {
        let func = self.m.get_function_ref_as_mut(func_id);
        func.insert_inst(self.insert_bb.unwrap(), inst);
    }

    fn give_module(self) -> LLVMModule {
        self.m
    }

    fn consume_cur_register(&mut self) -> LLVMValue {
        let reg_str = LLVMString::from(format!("{}", self.consume_label()));
        LLVMValue::new_register(reg_str)
    }
    fn consume_label(&mut self) -> usize {
        let l = self.label;
        self.label += 1;
        l
    }
    fn set_label(&mut self, l: usize) {
        self.label = l;
    }

    fn entry_bb_is_empty(&mut self, llvm_func_id: FunctionId) -> bool {
        let func = self.m.get_function_ref_as_mut(llvm_func_id);
        func.entry_bb_is_empty()
    }
}

fn create_module(build_option: &option::BuildOption) -> LLVMModule {
    let mut m: LLVMModule = Default::default();

    let arch = match build_option.arch {
        option::Architecture::X86_64 => target_triple::Arch::X86_64,
    };

    let tt = target_triple::TargetTriple {
        architecture: arch,
        sub: None,
        vendor: target_triple::Vendor::PC,
        sys: target_triple::Sys::LINUX,
        abi: target_triple::ABI::GNU,
    };
    m.set_target_triple(tt);

    let tl = target_datalayout::TargetDataLayout {
        endian: Some(target_datalayout::Endian::LITTLE),
        mangling: Some(target_datalayout::Mangling::ELF),
        integer_alignment: Some(target_datalayout::IntegerAlignment {
            size: 64,
            abi: 64,
            pref: None,
        }),
        float_alignment: Some(target_datalayout::FloatAlignment {
            size: target_datalayout::FloatAlignmentSize::LONGDOUBLE80,
            abi: 128,
            pref: None,
        }),
        native_integer_width: Some(target_datalayout::NativeIntegerWidth {
            sizes: vec![8, 16, 32, 64],
        }),
    };

    m.set_target_datalayout(tl);

    m
}
