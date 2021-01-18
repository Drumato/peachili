use std::{fs::File, io::prelude::*};

use crate::compiler::common::frontend;
use crate::{module, option};
use std::collections::HashMap;

use super::arch::x64;
pub fn compile_main<'a>(
    main_module: module::Module<'a>,
    build_option: option::BuildOption,
) -> Result<(), Box<dyn std::error::Error + 'a>> {
    let target = build_option.target;
    let (ast_roots, raw_type_env) = frontend::main(main_module, build_option)?;

    match target {
        option::Target::X86_64 => {
            // resolve_type
            let mut resolved_type_env: HashMap<String, x64::PeachiliType> = Default::default();

            // プリミティブな型はそのまま変換するだけ
            resolved_type_env.insert("Int64".to_string(), x64::PeachiliType::Int64);
            resolved_type_env.insert("Uint64".to_string(), x64::PeachiliType::Uint64);
            resolved_type_env.insert("Noreturn".to_string(), x64::PeachiliType::Noreturn);
            resolved_type_env.insert("ConstStr".to_string(), x64::PeachiliType::ConstStr);
            resolved_type_env.insert("Boolean".to_string(), x64::PeachiliType::Boolean);

            for ast_root in ast_roots.iter() {
                for common_tld in ast_root.decls.iter() {
                    match &common_tld.kind {
                        frontend::ast::TopLevelDeclKind::PubType { type_name, to: _ } => {
                            let (r, _) = x64::resolve_type(
                                &raw_type_env,
                                resolved_type_env,
                                &ast_root.module_name,
                                type_name,
                            )?;
                            resolved_type_env = r;
                        }
                        frontend::ast::TopLevelDeclKind::Function {
                            func_name,
                            return_type,
                            parameters,
                            stmts: _,
                        } => {
                            // 関数の返り値の型解決
                            let (r, return_type) = x64::resolve_type(
                                &raw_type_env,
                                resolved_type_env,
                                &ast_root.module_name,
                                return_type,
                            )?;
                            resolved_type_env = r;
                            resolved_type_env.insert(func_name.to_string(), return_type.clone());

                            for param in parameters.iter() {
                                let (r, _param_type) = x64::resolve_type(
                                    &raw_type_env,
                                    resolved_type_env,
                                    &ast_root.module_name,
                                    param.1,
                                )?;
                                resolved_type_env = r;
                            }
                        }
                        frontend::ast::TopLevelDeclKind::Import { module_name: _ } => {}
                        frontend::ast::TopLevelDeclKind::PubConst {
                            const_name: _,
                            const_type: _,
                            expr: _,
                        } => {}
                    }
                }
            }

            let x64_assembly_file = ast_roots
                .iter()
                .map(|root| {
                    x64::codegen_main(x64::ast_to_lower(root, resolved_type_env.clone()).unwrap())
                })
                .collect::<Vec<String>>()
                .join("\n# translation unit's end\n");

            let mut file = File::create("asm.s")?;
            write!(file, "{}", x64_assembly_file)?;
            file.flush()?;
        }
    }

    Ok(())
}
