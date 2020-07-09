use crate::common::ast::function;
use indexmap::map::IndexMap;

/// Root
#[derive(Debug, Clone)]
pub struct ASTRoot {
    pub funcs: Vec<function::FnId>,
    pub typedefs: IndexMap<String, StructDef>,
    pub alias: IndexMap<String, String>,
    pub parent_modules: Vec<String>,
}

impl Default for ASTRoot {
    fn default() -> Self {
        Self {
            funcs: Vec::new(),
            parent_modules: Vec::new(),
            alias: IndexMap::new(),
            typedefs: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub members: IndexMap<String, String>,
}

impl Default for StructDef {
    fn default() -> Self {
        Self {
            members: IndexMap::new(),
        }
    }
}
