use crate::common::position as pos;
use crate::compiler::resource as res;

#[allow(dead_code)]
pub struct PFunction {
    name: String,
    stack_offset: usize,
    position: pos::Position,
    stmts: Vec<res::StatementNode>,
    // locals: BTreeMap<String, res::PVariable>
    return_type: res::PType,
    // args: Vec<String>
}

impl PFunction {
    pub fn new(
        func_name: String,
        ptype: res::PType,
        def_pos: pos::Position,
        statements: Vec<res::StatementNode>,
    ) -> Self {
        Self {
            name: func_name,
            stack_offset: 0,
            position: def_pos,
            stmts: statements,
            return_type: ptype,
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
