use crate::common::three_address_code::value_kind;
use id_arena::Id;

pub type ValueId = Id<Value>;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Value {
    pub kind: value_kind::ValueKind,
}