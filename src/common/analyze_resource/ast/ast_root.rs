use crate::common::ast::function;
use std::collections::{BTreeMap, HashSet};

/// Root
#[derive(Debug, Clone)]
pub struct ASTRoot {
    pub funcs: Vec<function::FnId>,
    pub typedefs: BTreeMap<String, StructDef>,
    pub alias: BTreeMap<String, String>,
    pub called_functions: HashSet<String>,

    /// 定数名 => (型名, 代入されている式)
    pub constants: BTreeMap<String, (String, String)>,
    pub enum_decls: BTreeMap<String, EnumDef>,
}

impl Default for ASTRoot {
    fn default() -> Self {
        Self {
            funcs: Vec::new(),
            alias: BTreeMap::new(),
            typedefs: BTreeMap::new(),
            called_functions: HashSet::new(),
            constants: BTreeMap::new(),
            enum_decls: BTreeMap::new(),
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
        self.constants.append(&mut target.constants);
        self.enum_decls.append(&mut target.enum_decls);
        self.alias.append(&mut target.alias);
        self.called_functions = &self.called_functions | &target.called_functions;
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

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub variants: BTreeMap<String, VariantDef>,
}

#[derive(Debug, Clone)]
pub struct VariantDef {
    pub tag: usize,
}
