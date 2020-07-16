use crate::common::option::Target;
use std::collections::BTreeMap;

/// 型
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Type {
    /// 型の種類
    pub kind: TypeKind,
    /// 型のサイズ
    pub size: usize,
}

impl Type {
    /// Int64型サイズ
    pub fn int64_size(target: Target) -> usize {
        match target {
            Target::X86_64 => 8,
        }
    }

    /// Uint64型サイズ
    pub fn uint64_size(target: Target) -> usize {
        match target {
            Target::X86_64 => 8,
        }
    }

    /// ポインタ型サイズ
    pub fn pointer_size(target: Target) -> usize {
        match target {
            Target::X86_64 => 8,
        }
    }
    /// Boolean型サイズ
    pub fn boolean_size(target: Target) -> usize {
        match target {
            Target::X86_64 => 8,
        }
    }
    /// ConstStr
    pub fn conststr_size(target: Target) -> usize {
        match target {
            Target::X86_64 => 8,
        }
    }

    /// Int64型を新たに割り当てる
    pub fn new_int64(size: usize) -> Self {
        Self {
            kind: TypeKind::INT64,
            size,
        }
    }

    /// Uint64型を新たに割り当てる
    pub fn new_uint64(size: usize) -> Self {
        Self {
            kind: TypeKind::UINT64,
            size,
        }
    }
    /// Boolean型を新たに割り当てる
    pub fn new_boolean(size: usize) -> Self {
        Self {
            kind: TypeKind::BOOLEAN,
            size,
        }
    }
    /// Noreturn型
    pub fn new_noreturn() -> Self {
        Self {
            kind: TypeKind::NORETURN,
            size: 0,
        }
    }
    /// ConstStr
    pub fn new_const_str(size: usize) -> Self {
        Self {
            kind: TypeKind::CONSTSTR,
            size,
        }
    }

    /// ポインタ型を新たに割り当てる
    pub fn new_pointer(to: Self, size: usize) -> Self {
        Self {
            kind: TypeKind::POINTER {
                to: Box::new(to),
            },
            size,
        }
    }

    /// 構造体型型を新たに割り当てる
    pub fn new_struct(members: BTreeMap<String, (Box<Type>, usize)>, total_size: usize) -> Self {
        Self {
            kind: TypeKind::STRUCT {
                members
            },
            size: total_size,
        }
    }

    /// 構造体型であるか
    pub fn is_struct(&self) -> bool {
        match self.kind {
            TypeKind::STRUCT { members: _ } => true,
            _ => false,
        }
    }

    /// 構造体型であると解釈し, メンバを取り出す
    pub fn get_members(&self) -> &BTreeMap<String, (Box<Type>, usize)> {
        match &self.kind {
            TypeKind::STRUCT { members } => members,
            _ => panic!("cannot call get_members() with not a struct"),
        }
    }
}


/// 型の種類
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum TypeKind {
    /// 64bit整数
    INT64,
    /// 64bit非符号付き整数
    UINT64,
    /// ポインタ
    POINTER {
        to: Box<Type>,
    },
    /// ConstStr
    CONSTSTR,
    /// Boolean
    BOOLEAN,
    /// Noreturn
    NORETURN,
    /// 構造体型
    STRUCT {
        /// member_name -> (member_type, member_offset)
        members: BTreeMap<String, (Box<Type>, usize)>
    },
}