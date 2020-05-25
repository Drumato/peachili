use std::collections::BTreeMap;

use crate::common::position as pos;
use crate::compiler::general::resource as res;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
#[allow(dead_code)]
pub struct PFunction {
    name: String,
    stack_offset: usize,
    position: pos::Position,
    stmts: Vec<res::StatementNode>,
    pub locals: BTreeMap<String, res::PVariable>,
    pub strings: BTreeMap<String, u64>,
    return_type: res::PType,
    args: Vec<String>,

    module_path: String,
}

impl PFunction {
    pub fn new(
        func_name: String,
        ptype: res::PType,
        arg_names: Vec<String>,
        def_pos: pos::Position,
        path: String,
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
        }
    }

    pub fn arg_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn collect_arg_types(&self, type_map: &BTreeMap<String, res::PType>) -> Vec<res::PType> {
        let locals = self.get_locals();
        let args = self.get_args();

        let mut arg_types: Vec<res::PType> = Vec::new();

        for arg in args.iter() {
            if let Some(pvar) = locals.get(arg) {
                if let res::PTypeKind::UNRESOLVED(name) = &pvar.get_type().kind {
                    let type_last = res::IdentName::last_name(name);
                    arg_types.push(type_map.get(&type_last).unwrap().clone());

                    continue;
                }
            }

            let arg_type = locals.get(arg).unwrap().get_type();
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

    pub fn add_local(&mut self, name: String, pvar: res::PVariable) {
        if let Some(_old_var) = self.locals.insert(name, pvar) {
            panic!("detected duplicated variable declaration in {}", self.name);
        }
    }
    pub fn add_string(&mut self, contents: String, hash: u64) {
        self.strings.insert(contents, hash);
    }

    pub fn get_return_type(&self) -> &res::PType {
        &self.return_type
    }
    pub fn get_statements(&self) -> &Vec<res::StatementNode> {
        &self.stmts
    }
    pub fn get_args(&self) -> &Vec<String> {
        &self.args
    }

    pub fn get_locals(&self) -> &BTreeMap<String, res::PVariable> {
        &self.locals
    }
    pub fn set_locals(&mut self, locals: BTreeMap<String, res::PVariable>) {
        self.locals = locals;
    }
    pub fn get_strings(&self) -> &BTreeMap<String, u64> {
        &self.strings
    }
    pub fn get_stack_offset(&self) -> usize {
        self.stack_offset
    }
    pub fn set_stack_offset(&mut self, offset: usize) {
        self.stack_offset = offset;
    }

    pub fn copy_func_name(&self) -> String {
        self.name.clone()
    }
}

impl std::fmt::Display for PFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(
            f,
            "{} {}() {} {{",
            self.position, self.name, self.return_type
        )?;

        for st in self.stmts.iter() {
            writeln!(f, "\t{}", st)?;
        }
        write!(f, "}}")
    }
}
