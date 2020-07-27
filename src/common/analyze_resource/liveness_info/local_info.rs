use crate::common::three_address_code as tac;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub struct LocalLivenessInfo {
    /// 各IR時点で生成されるTemp変数の集合
    pub defs: BTreeMap<tac::CodeId, BTreeSet<tac::ValueId>>,
    /// 各IR時点で使用されるTemp変数の集合
    pub uses: BTreeMap<tac::CodeId, BTreeSet<tac::ValueId>>,
    /// 入口生存の集合
    pub live_in: BTreeMap<tac::CodeId, BTreeSet<tac::ValueId>>,
    /// 出口生存の集合
    pub live_out: BTreeMap<tac::CodeId, BTreeSet<tac::ValueId>>,
}

impl Default for LocalLivenessInfo {
    fn default() -> Self {
        Self {
            defs: BTreeMap::new(),
            uses: BTreeMap::new(),
            live_in: BTreeMap::new(),
            live_out: BTreeMap::new(),
        }
    }
}

impl LocalLivenessInfo {
    pub fn get_live_in(&self, code_id: &tac::CodeId) -> &BTreeSet<tac::ValueId> {
        self.live_in.get(code_id).unwrap()
    }
    pub fn get_live_out(&self, code_id: &tac::CodeId) -> &BTreeSet<tac::ValueId> {
        self.live_out.get(code_id).unwrap()
    }
}