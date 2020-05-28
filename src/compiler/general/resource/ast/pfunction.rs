use std::collections::BTreeMap;

use crate::common::{module, position as pos};
use crate::compiler::general::resource as res;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
#[allow(dead_code)]
pub struct PFunction {
    name: res::PStringId,
    stack_offset: usize,
    position: pos::Position,
    stmts: Vec<res::StatementNode>,
    pub locals: BTreeMap<Vec<res::PStringId>, res::PVariable>,
    pub strings: BTreeMap<res::PStringId, u64>,
    return_type: res::PType,
    args: Vec<res::PStringId>,

    module_path: String,
    pub module_id: module::ModuleId,
}

impl PFunction {
    pub fn new(
        func_name: res::PStringId,
        ptype: res::PType,
        arg_names: Vec<res::PStringId>,
        def_pos: pos::Position,
        path: String,
        mod_id: module::ModuleId,
    ) -> Self {
        Self {
            name: func_name,
            stack_offset: 0,
            position: def_pos,
            stmts: Vec::new(),
            locals: BTreeMap::new(),
            strings: BTreeMap::new(),
            return_type: ptype,
            args: arg_names,
            module_path: path,
            module_id: mod_id,
        }
    }

    pub fn arg_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn collect_arg_types(
        &self,
        type_map: &BTreeMap<res::PStringId, res::PType>,
    ) -> Vec<res::PType> {
        let locals = self.get_locals();
        let args = self.get_args();

        let mut arg_types: Vec<res::PType> = Vec::new();

        for arg in args.iter() {
            if let Some(pvar) = locals.get(vec![*arg].as_slice()) {
                if let res::PTypeKind::UNRESOLVED(name) = &pvar.get_type().kind {
                    let type_last = res::IdentName::last_name(name);
                    arg_types.push(type_map.get(&type_last).unwrap().clone());

                    continue;
                }
            }

            let arg_type = locals.get(vec![*arg].as_slice()).unwrap().get_type();
            arg_types.push(arg_type.clone());
        }

        arg_types
    }

    pub fn copy_module_path(&self) -> String {
        self.module_path.to_string()
    }
    pub fn replace_statements(&mut self, stmts: Vec<res::StatementNode>) {
        self.stmts = stmts;
    }

    pub fn add_local(&mut self, name_ids: Vec<res::PStringId>, pvar: res::PVariable) {
        if let Some(_old_var) = self.locals.insert(name_ids, pvar) {
            panic!("detected duplicated variable declaration");
        }
    }
    pub fn add_string(&mut self, contents: res::PStringId, hash: u64) {
        self.strings.insert(contents, hash);
    }

    pub fn get_return_type(&self) -> &res::PType {
        &self.return_type
    }
    pub fn get_statements(&self) -> &Vec<res::StatementNode> {
        &self.stmts
    }
    pub fn get_args(&self) -> &Vec<res::PStringId> {
        &self.args
    }

    pub fn get_locals(&self) -> &BTreeMap<Vec<res::PStringId>, res::PVariable> {
        &self.locals
    }
    pub fn set_locals(&mut self, locals: BTreeMap<Vec<res::PStringId>, res::PVariable>) {
        self.locals = locals;
    }
    pub fn get_strings(&self) -> &BTreeMap<res::PStringId, u64> {
        &self.strings
    }
    pub fn get_stack_offset(&self) -> usize {
        self.stack_offset
    }
    pub fn set_stack_offset(&mut self, offset: usize) {
        self.stack_offset = offset;
    }

    pub fn get_func_name_id(&self) -> res::PStringId {
        self.name
    }
}

impl std::fmt::Display for PFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "{} {:?}() {} {{",
            self.position, self.name, self.return_type
        )?;

        for st in self.stmts.iter() {
            writeln!(f, "\t{}", st)?;
        }
        write!(f, "}}")
    }
}
