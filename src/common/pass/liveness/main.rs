use std::collections::{BTreeMap, BTreeSet};

use crate::common::{
    liveness_info as li,
    three_address_code as tac,
    cfg,
};
use crate::common::three_address_code::CodeKind;

pub fn liveness_analysis(
    ir_module: &tac::IRModule,
    local_cfg: &BTreeMap<tac::IRFunctionId, cfg::LocalControlFlowGraph>,
) -> BTreeMap<tac::IRFunctionId, li::LocalLivenessInfo> {
    let mut liveness_information = BTreeMap::new();

    for fn_id in ir_module.funcs.iter() {
        let ir_fn = ir_module.fn_allocator.get(*fn_id).unwrap();

        // まずはuses, defs集合を求める
        let mut info_in_fn = setup_info_in_fn(ir_fn);

        // 実際に解析を行う
        info_in_fn.analyze(*fn_id, ir_fn, local_cfg);

        liveness_information.insert(*fn_id, info_in_fn);
    }

    liveness_information
}

fn setup_info_in_fn(ir_fn: &tac::IRFunction) -> li::LocalLivenessInfo {
    let mut info: li::LocalLivenessInfo = Default::default();

    for code_id in ir_fn.codes.iter() {
        let mut defs = BTreeSet::new();
        let mut uses = BTreeSet::new();

        let code = ir_fn.code_allocator.lock().unwrap().get(*code_id).unwrap().clone();
        setup_info_ir(ir_fn, code, &mut defs, &mut uses);

        info.defs.insert(*code_id, defs);
        info.uses.insert(*code_id, uses);
    }

    info
}

fn setup_info_ir(ir_fn: &tac::IRFunction, code: tac::Code, defs: &mut BTreeSet<tac::ValueId>, uses: &mut BTreeSet<tac::ValueId>) {
    match &code.kind {
        CodeKind::JUMPIFFALSE { label: _, cond_result } => {
            if ir_fn.get_value(*cond_result).is_temp() {
                uses.insert(*cond_result);
            }
        }
        CodeKind::ADDRESSOF { value, result }
        | CodeKind::NEG { value, result }
        | CodeKind::DEREFERENCE { value, result }
        | CodeKind::ASSIGN { value, result } => {
            if ir_fn.get_value(*value).is_temp() {
                uses.insert(*value);
            }
            if ir_fn.get_value(*result).is_temp() {
                defs.insert(*result);
            }
        }

        CodeKind::ADD { lop, rop, result }
        | CodeKind::SUB { lop, rop, result }
        | CodeKind::MUL { lop, rop, result }
        | CodeKind::DIV { lop, rop, result } => {
            if ir_fn.get_value(*lop).is_temp() {
                uses.insert(*lop);
            }
            if ir_fn.get_value(*rop).is_temp() {
                uses.insert(*rop);
            }
            if ir_fn.get_value(*result).is_temp() {
                defs.insert(*result);
            }
        }
        CodeKind::ALLOC { temp } => {
            if ir_fn.get_value(*temp).is_temp() {
                defs.insert(*temp);
            }
        }
        CodeKind::PARAM { value } => {
            if ir_fn.get_value(*value).is_temp() {
                uses.insert(*value);
            }
        }
        CodeKind::RETURN { value } => {
            if ir_fn.get_value(*value).is_temp() {
                uses.insert(*value);
            }
        }
        CodeKind::MEMBER { id: _, member: _, result } => {
            if ir_fn.get_value(*result).is_temp() {
                defs.insert(*result);
            }
        }
        _ => {}
    }
}

impl li::LocalLivenessInfo {
    fn analyze(
        &mut self,
        fn_id: tac::IRFunctionId,
        ir_fn: &tac::IRFunction,
        local_cfg: &BTreeMap<tac::IRFunctionId, cfg::LocalControlFlowGraph>,
    ) {
        // 空のSetを作っておく
        for code_id in ir_fn.codes.iter() {
            self.live_in.insert(*code_id, BTreeSet::new());
            self.live_out.insert(*code_id, BTreeSet::new());
        }

        // repeat
        'outer: loop {
            // 変更をチェックするための集合
            let mut in_dash: BTreeMap<tac::CodeId, BTreeSet<tac::ValueId>> = BTreeMap::new();
            let mut out_dash: BTreeMap<tac::CodeId, BTreeSet<tac::ValueId>> = BTreeMap::new();

            // foreach n
            for code_id in ir_fn.codes.iter().rev() {
                // 後で変更が無いかチェックする為に保存
                in_dash.insert(*code_id, self.live_in.get(code_id).unwrap().clone());
                out_dash.insert(*code_id, self.live_out.get(code_id).unwrap().clone());

                // in[n] <- use[n] ∪ (out[n] - def[n])
                *self.live_in.get_mut(code_id).unwrap() =
                    self.uses.get(code_id).unwrap() |
                        &(self.live_out.get(code_id).unwrap() - self.defs.get(code_id).unwrap());

                // out[n] <- U in[s] (where s ∈ succ[n])
                for successor in local_cfg.get(&fn_id).unwrap().get_successors(code_id).iter() {
                    *self.live_out.get_mut(code_id).unwrap()
                        = self.live_out.get(code_id).unwrap() |
                        self.live_in.get(successor).unwrap();
                }
            }

            // until in'[n] == in[n] and out'[n] == out[n] for all n
            let mut chg_flg: bool = true;
            for (in_code, in_set) in self.live_in.iter() {
                if in_set != in_dash.get(in_code).unwrap() {
                    chg_flg = false;
                }
                if self.live_out.get(in_code).unwrap() != out_dash.get(in_code).unwrap() {
                    chg_flg = false;
                }
            }

            if chg_flg {
                break 'outer;
            }
        }
    }
}