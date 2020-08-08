use crate::common::option::Target;
use crate::common::peachili_type::Type;
use crate::common::three_address_code::value_kind;
use id_arena::Id;

pub type ValueId = Id<Value>;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Value {
    pub kind: value_kind::ValueKind,
    pub ty: Type,
}

#[allow(dead_code)]
impl Value {
    pub fn dump(&self) -> String {
        format!("<{} {}>", self.ty.dump(), self.kind.dump())
    }

    pub fn new(k: value_kind::ValueKind, ty: Type) -> Self {
        Self { kind: k, ty }
    }

    pub fn new_int64(value: i64, target: Target) -> Self {
        Self::new(
            value_kind::ValueKind::INTLITERAL { value },
            Type::new_int64(target),
        )
    }
    pub fn new_uint64(value: u64, target: Target) -> Self {
        Self::new(
            value_kind::ValueKind::UINTLITERAL { value },
            Type::new_uint64(target),
        )
    }
    pub fn new_boolean(truth: bool, target: Target) -> Self {
        Self::new(
            value_kind::ValueKind::BOOLEANLITERAL { truth },
            Type::new_boolean(target),
        )
    }
    pub fn new_string_literal(contents: String, target: Target) -> Self {
        Self::new(
            value_kind::ValueKind::STRINGLITERAL { contents },
            Type::new_const_str(target),
        )
    }
    pub fn new_temp(number: usize, ty: Type) -> Self {
        Self::new(value_kind::ValueKind::TEMP { number }, ty)
    }

    pub fn is_temp(&self) -> bool {
        match &self.kind {
            value_kind::ValueKind::TEMP { number: _ } => true,
            _ => false,
        }
    }
    pub fn is_id(&self) -> bool {
        match &self.kind {
            value_kind::ValueKind::ID { name: _ } => true,
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
