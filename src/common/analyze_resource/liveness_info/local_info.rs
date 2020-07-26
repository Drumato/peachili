use crate::common::three_address_code as tac;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

pub struct LocalLivenessInfo {
    /// 各IR時点で生成されるTemp変数の集合
    pub defs: BTreeMap<tac::CodeId, BTreeSet<tac::ValueId>>,
    /// 各IR時点で使用されるTemp変数の集合
    pub uses: BTreeMap<tac::CodeId, BTreeSet<tac::ValueId>>,
}

impl Default for LocalLivenessInfo {
    fn default() -> Self {
        Self {
            defs: BTreeMap::new(),
            uses: BTreeMap::new(),
        }
    }
}