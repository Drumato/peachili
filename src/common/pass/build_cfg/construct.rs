use crate::common::three_address_code as tac;
use tac::CodeKind;
use crate::common::cfg::LocalControlFlowGraph;
use std::collections::{BTreeMap, BTreeSet};

pub fn build_local_cfg(ir_module: &tac::IRModule) -> BTreeMap<tac::IRFunctionId, LocalControlFlowGraph> {
    let mut local_cfg = BTreeMap::new();

    for fn_id in ir_module.funcs.iter() {
        let ir_fn = ir_module.fn_allocator.get(*fn_id).unwrap();

        let cfg_in_func = build_graph_in_func(ir_fn);
        local_cfg.insert(*fn_id, cfg_in_func);
    }

    local_cfg
}

fn build_graph_in_func(ir_fn: &tac::IRFunction) -> LocalControlFlowGraph {
    let mut graph: LocalControlFlowGraph = Default::default();

    let mut label_to_jmp: BTreeMap<String, tac::CodeId> = BTreeMap::new();
    // 複数の可能性がある
    let mut jmp_to_label: BTreeMap<String, BTreeSet<tac::CodeId>> = BTreeMap::new();

    for (idx, code_id) in ir_fn.codes.iter().enumerate() {
        if let Ok(arena) = ir_fn.code_allocator.lock() {
            let code = arena.get(*code_id).unwrap();

            match &code.kind {
                CodeKind::LABEL { name } => {
                    label_to_jmp.insert(name.clone(), *code_id);

                    if let Some(jump_codes) = jmp_to_label.get(name) {
                        for jump_code in jump_codes.iter() {
                            add_pred_edge(&mut graph, *code_id, *jump_code);
                            add_succ_edge(&mut graph, *jump_code, *code_id);
                        }
                    }

                    add_pred_edge_from(&mut graph, *code_id, &ir_fn.codes, idx);
                    add_succ_edge_from(&mut graph, *code_id, &ir_fn.codes, idx);
                }
                CodeKind::JUMPIFFALSE { label, cond_result: _ } => {
                    if !jmp_to_label.contains_key(label) {
                        jmp_to_label.insert(label.clone(), BTreeSet::new());
                    }
                    jmp_to_label.get_mut(label).unwrap().insert(*code_id);

                    if let Some(label_code) = label_to_jmp.get(label) {
                        add_succ_edge(&mut graph, *code_id, *label_code);
                        add_pred_edge(&mut graph, *label_code, *code_id);
                    }

                    add_succ_edge_from(&mut graph, *code_id, &ir_fn.codes, idx);
                    add_pred_edge_from(&mut graph, *code_id, &ir_fn.codes, idx);
                }
                CodeKind::JUMP { label } => {
                    if !jmp_to_label.contains_key(label) {
                        jmp_to_label.insert(label.clone(), BTreeSet::new());
                    }
                    jmp_to_label.get_mut(label).unwrap().insert(*code_id);

                    if let Some(label_code) = label_to_jmp.get(label) {
                        add_succ_edge(&mut graph, *code_id, *label_code);
                        add_pred_edge(&mut graph, *label_code, *code_id);
                    }
                    add_pred_edge_from(&mut graph, *code_id, &ir_fn.codes, idx);
                }
                _ => {
                    add_pred_edge_from(&mut graph, *code_id, &ir_fn.codes, idx);
                    add_succ_edge_from(&mut graph, *code_id, &ir_fn.codes, idx);
                }
            }
        }
    }

    graph
}

fn add_pred_edge_from(graph: &mut LocalControlFlowGraph, src: tac::CodeId, codes: &[tac::CodeId], idx: usize) {
    if idx > 0 {
        graph.add_predecessor(src, codes[idx - 1]);
    }
}

fn add_succ_edge_from(graph: &mut LocalControlFlowGraph, src: tac::CodeId, codes: &[tac::CodeId], idx: usize) {
    if idx < codes.len() - 1 {
        graph.add_successor(src, codes[idx + 1]);
    }
}

fn add_pred_edge(graph: &mut LocalControlFlowGraph, src: tac::CodeId, dst: tac::CodeId) {
    graph.add_predecessor(src, dst);
}

fn add_succ_edge(graph: &mut LocalControlFlowGraph, src: tac::CodeId, dst: tac::CodeId) {
    graph.add_successor(src, dst)
}