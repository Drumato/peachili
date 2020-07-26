use crate::common::{
    three_address_code as tac,
    liveness_info as li,
};
use std::collections::{BTreeMap, BTreeSet};
use crate::common::three_address_code::CodeKind;

pub fn liveness_analysis(ir_module: &tac::IRModule) -> BTreeMap<tac::IRFunctionId, li::LocalLivenessInfo> {
    let mut liveness_information = BTreeMap::new();

    for fn_id in ir_module.funcs.iter() {
        let ir_fn = ir_module.fn_allocator.get(*fn_id).unwrap();

        // まずはuses, defs集合を求める
        let info_in_fn = setup_info_in_fn(ir_fn);
        liveness_information.insert(*fn_id, info_in_fn);

        // 実際に解析を行う
    }

    liveness_information
}

fn setup_info_in_fn(ir_fn: &tac::IRFunction) -> li::LocalLivenessInfo {
    let mut info: li::LocalLivenessInfo = Default::default();

    for code_id in ir_fn.codes.iter() {
        let mut defs = BTreeSet::new();
        let mut uses = BTreeSet::new();

        let code = ir_fn.code_allocator.lock().unwrap().get(*code_id).unwrap().clone();
        setup_info_ir(code, &mut defs, &mut uses);

        info.defs.insert(*code_id, defs);
        info.uses.insert(*code_id, uses);
    }

    info
}

fn setup_info_ir(code: tac::Code, defs: &mut BTreeSet<tac::ValueId>, uses: &mut BTreeSet<tac::ValueId>) {
    match &code.kind {
        CodeKind::JUMPIFFALSE { label: _, cond_result } => {
            uses.insert(*cond_result);
        }
        CodeKind::ADDRESSOF { value, result }
        | CodeKind::NEG { value, result }
        | CodeKind::DEREFERENCE { value, result }
        | CodeKind::ASSIGN { value, result } => {
            uses.insert(*value);
            defs.insert(*result);
        }

        CodeKind::ADD { lop, rop, result }
        | CodeKind::SUB { lop, rop, result }
        | CodeKind::MUL { lop, rop, result }
        | CodeKind::DIV { lop, rop, result } => {
            uses.insert(*lop);
            uses.insert(*rop);
            defs.insert(*result);
        }
        CodeKind::ALLOC { temp } => {
            defs.insert(*temp);
        }
        CodeKind::PARAM { value } => {
            uses.insert(*value);
        }
        CodeKind::RETURN { value } => {
            uses.insert(*value);
        }
        CodeKind::MEMBER { id: _, member: _, result } => {
            defs.insert(*result);
        }
        _ => {}
    }
}