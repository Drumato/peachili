use crate::common::ast::function;
use std::collections::BTreeMap;

/// Root
#[derive(Debug, Clone)]
pub struct ASTRoot {
    pub funcs: Vec<function::FnId>,
    pub typedefs: BTreeMap<String, StructDef>,
    pub alias: BTreeMap<String, String>,
}

impl Default for ASTRoot {
    fn default() -> Self {
        Self {
            funcs: Vec::new(),
            alias: BTreeMap::new(),
            typedefs: BTreeMap::new(),
        }
    }
}

impl ASTRoot {
    /// 別モジュールのASTRootを吸収する
    pub fn absorb(&mut self, mut target: Self) {
        let dst_func_number = self.funcs.len();
        let src_func_number = target.funcs.len();
        self.funcs.append(&mut target.funcs);
        assert_eq!(dst_func_number + src_func_number, self.funcs.len());

        self.typedefs.append(&mut target.typedefs);
        self.alias.append(&mut target.alias);
    }
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub members: BTreeMap<String, String>,
}

impl Default for StructDef {
    fn default() -> Self {
        Self {
            members: BTreeMap::new(),
        }
    }
}
