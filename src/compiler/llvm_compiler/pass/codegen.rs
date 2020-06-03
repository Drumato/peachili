use llvm_scratch::core::{module::Module as LLVMModule, target_datalayout, target_triple};

use crate::common::{module, option};
use crate::compiler::general::resource as res;

pub fn codegen(
    build_option: &option::BuildOption,
    ast_root: res::ASTRoot,
    module_allocator: &module::ModuleAllocator,
) -> llvm_scratch::core::module::Module {
    let generator = Generator {
        m: create_module(build_option),
        label: 0,
    };

    let functions = ast_root.get_functions();

    for (_func_name, func) in functions.iter() {
        let _const_pool = module_allocator
            .get_module_ref(&func.module_id)
            .unwrap()
            .get_const_pool_ref();
    }

    generator.give_module()
}

struct Generator {
    m: LLVMModule,
    label: usize,
}

#[allow(dead_code)]
impl Generator {
    fn give_module(self) -> LLVMModule {
        self.m
    }
    fn consume_label(&mut self) -> usize {
        let l = self.label;
        self.label += 1;
        l
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
