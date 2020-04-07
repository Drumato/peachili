use std::collections::BTreeMap;

use crate::common::position as pos;
use crate::compiler::resource as res;

#[derive(Clone)]
#[allow(dead_code)]
pub struct PFunction {
    name: String,
    stack_offset: usize,
    position: pos::Position,
    stmts: Vec<res::StatementNode>,
    locals: BTreeMap<String, res::PVariable>,
    return_type: res::PType,
    // args: Vec<String>
}

impl PFunction {
    pub fn new(func_name: String, ptype: res::PType, def_pos: pos::Position) -> Self {
        Self {
            name: func_name,
            stack_offset: 0,
            position: def_pos,
            stmts: Vec::new(),
            locals: BTreeMap::new(),
            return_type: ptype,
        }
    }

    pub fn replace_statements(&mut self, stmts: Vec<res::StatementNode>) {
        self.stmts = stmts;
    }

    pub fn add_local(&mut self, name: String, pvar: res::PVariable) {
        if let Some(_old_var) = self.locals.insert(name, pvar) {
            panic!("detected duplicated variable declaration in {}", self.name);
        }
    }
}

impl std::fmt::Display for PFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {}() {} {{\n",
            self.position, self.name, self.return_type
        )?;

        for st in self.stmts.iter() {
            write!(f, "\t{}\n", st)?;
        }
        write!(f, "}}")
    }
}
