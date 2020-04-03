use crate::common::module;
use crate::common::position as pos;
use crate::compiler::resource as res;

pub struct PFunction {
    name: String,
    stack_offset: usize,
    position: pos::Position,
    // stmts: Vec<StatementNode>,
    // locals: BTreeMap<String, res::PVariable>
    // return_type: res::PType,
    // args: Vec<String>
}

impl module::Functionable for PFunction {
    fn get_name(&self) -> &String {
        &self.name
    }
    fn def_position(&self) -> &pos::Position {
        &self.position
    }
}
