use crate::common::three_address_code as tac;
use std::collections::{BTreeMap, BTreeSet};

/// BasicBlockを考慮しない，関数内のflat-graph
pub struct LocalControlFlowGraph {
    /// あるノードから伸びる先行節
    pub predecessors: BTreeMap<tac::CodeId, BTreeSet<tac::CodeId>>,
    /// あるノードから伸びる後続節
    pub successors: BTreeMap<tac::CodeId, BTreeSet<tac::CodeId>>,
}

impl LocalControlFlowGraph {
    pub fn add_predecessor(&mut self, src: tac::CodeId, dst: tac::CodeId) {
        self.predecessors.entry(src).or_insert_with(BTreeSet::new);
        self.predecessors.get_mut(&src).unwrap().insert(dst);
    }

    pub fn add_successor(&mut self, src: tac::CodeId, dst: tac::CodeId) {
        self.successors.entry(src).or_insert_with(BTreeSet::new);
        self.successors.get_mut(&src).unwrap().insert(dst);
    }

    pub fn get_successors(&self, code_id: &tac::CodeId) -> &BTreeSet<tac::CodeId> {
        self.predecessors.get(code_id).unwrap()
    }
}

impl Default for LocalControlFlowGraph {
    fn default() -> Self {
        Self {
            predecessors: BTreeMap::new(),
            successors: BTreeMap::new(),
        }
    }
}
