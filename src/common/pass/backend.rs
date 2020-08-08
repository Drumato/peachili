use crate::common::{analyze_resource as ar, option, pass, three_address_code as tac};
use crate::debug;
use colored::*;

use std::collections::BTreeMap;

// 共通のバックエンドプロセス
pub fn backend(
    fn_arena: ar::ast::FnArena,
    ast_root: ar::ast::ASTRoot,
    type_env: &BTreeMap<String, BTreeMap<String, ar::peachili_type::Type>>,
    target: option::Target,
    verbose_ir: bool,
) -> (
    tac::IRModule,
    BTreeMap<tac::IRFunctionId, ar::cfg::LocalControlFlowGraph>,
) {
    let ir_module = pass::translate_ir(fn_arena, ast_root, type_env, target);

    if verbose_ir {
        eprintln!("{}", "dump HIR to 'hir_dump'...".bold().blue());
        debug::dump_hir(&ir_module);
        eprintln!("{}", "done!".bold().blue());
    }

    // BasicBlockのない，ローカルなグラフを作成する
    let local_cfg = pass::build_local_cfg(&ir_module);

    if verbose_ir {
        eprintln!("{}", "dump CFG to 'local_cfg.dot' ...".bold().blue());
        debug::dump_local_cfg("local_cfg.dot", &ir_module, &local_cfg);
        eprintln!("{}", "done!".bold().blue());
    }

    (ir_module, local_cfg)
}
