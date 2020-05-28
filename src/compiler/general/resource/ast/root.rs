use std::collections::BTreeMap;

use crate::compiler::general::resource as res;

pub struct ASTRoot {
    func_map: BTreeMap<res::PStringId, res::PFunction>,
    type_map: BTreeMap<res::PStringId, res::PType>,
}

impl Default for ASTRoot {
    fn default() -> Self {
        Self {
            func_map: BTreeMap::new(),
            type_map: BTreeMap::new(),
        }
    }
}

impl ASTRoot {
    pub fn add_typedef(&mut self, type_name_id: res::PStringId, src_type: res::PType) {
        assert!(self.type_map.insert(type_name_id, src_type).is_none());
    }
    pub fn add_pfunction(&mut self, name_id: res::PStringId, func: res::PFunction) {
        assert!(self.func_map.insert(name_id, func).is_none());
    }
    pub fn append(&mut self, mut sub_root: Self) {
        self.func_map.append(sub_root.get_mutable_functions());
        self.type_map.append(sub_root.get_mutable_typedefs());
    }
    pub fn append_functions(&mut self, extra: &mut BTreeMap<res::PStringId, res::PFunction>) {
        self.func_map.append(extra);
    }
    pub fn get_mutable_functions(&mut self) -> &mut BTreeMap<res::PStringId, res::PFunction> {
        &mut self.func_map
    }
    pub fn get_mutable_typedefs(&mut self) -> &mut BTreeMap<res::PStringId, res::PType> {
        &mut self.type_map
    }

    pub fn copy_functions(&self) -> BTreeMap<res::PStringId, res::PFunction> {
        self.func_map.clone()
    }
    pub fn copy_typedefs(&self) -> BTreeMap<res::PStringId, res::PType> {
        self.type_map.clone()
    }

    pub fn get_typedefs(&self) -> &BTreeMap<res::PStringId, res::PType> {
        &self.type_map
    }
    pub fn get_functions(&self) -> &BTreeMap<res::PStringId, res::PFunction> {
        &self.func_map
    }
    pub fn give_functions(self) -> BTreeMap<res::PStringId, res::PFunction> {
        self.func_map
    }

    pub fn add_local_var_to(
        &mut self,
        func_name_id: res::PStringId,
        var_name_ids: Vec<res::PStringId>,
        pvar: res::PVariable,
    ) {
        if let Some(p_func) = self.func_map.get_mut(&func_name_id) {
            p_func.add_local(var_name_ids, pvar);
        }
    }

    pub fn add_string_to(
        &mut self,
        func_name_id: res::PStringId,
        contents_id: res::PStringId,
        hash: u64,
    ) {
        if let Some(p_func) = self.func_map.get_mut(&func_name_id) {
            p_func.add_string(contents_id, hash);
        }
    }

    pub fn replace_statement(&mut self, name_id: res::PStringId, stmts: Vec<res::StatementNode>) {
        if let Some(p_func) = self.func_map.get_mut(&name_id) {
            p_func.replace_statements(stmts);
        }
    }
}
