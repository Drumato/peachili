use crate::common::three_address_code::value_kind;
use id_arena::Id;

pub type ValueId = Id<Value>;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Value {
    pub kind: value_kind::ValueKind,
}

#[allow(dead_code)]
impl Value {
    pub fn dump(&self) -> String {
        self.kind.dump()
    }

    pub fn is_temp(&self) -> bool {
        match &self.kind {
            value_kind::ValueKind::TEMP { number: _ } => true,
            _ => false,
        }
    }

    pub fn copy_contents(&self) -> String {
        match &self.kind {
            value_kind::ValueKind::STRINGLITERAL { contents } => contents.clone(),
            value_kind::ValueKind::ID { name } => name.clone(),
            _ => String::new(),
        }
    }

    pub fn get_virt_number(&self) -> usize {
        match &self.kind {
            value_kind::ValueKind::TEMP { number } => *number,
            _ => 0,
        }
    }
}