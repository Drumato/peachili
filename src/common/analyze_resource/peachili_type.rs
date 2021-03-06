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
    pub fn dump(&self) -> String {
        match &self.kind {
            TypeKind::BOOLEAN => "Boolean".to_string(),
            TypeKind::CONSTSTR => "ConstStr".to_string(),
            TypeKind::INT64 => "Int64".to_string(),
            TypeKind::UINT64 => "Uint64".to_string(),
            TypeKind::NORETURN => "Noreturn".to_string(),
            TypeKind::FUNCTION { return_type } => format!("func() {}", return_type.dump()),
            TypeKind::POINTER { to } => format!("*{}", to.dump()),
            TypeKind::STRUCT { members } => {
                let mut type_strs = Vec::new();

                for (member_name, (member_type, _offset)) in members.iter() {
                    type_strs.push(format!("{}: {}", member_name, member_type.dump()));
                }

                format!("{{ {} }}", type_strs.join(", "))
            }
            TypeKind::CONST {
                const_type,
                value: _,
            } => const_type.dump(),
            TypeKind::ENUM => "enum".to_string(),
        }
    }
    /// 関数型サイズ
    pub fn new_function(ret_ty: Type) -> Self {
        Self {
            kind: TypeKind::FUNCTION {
                return_type: Box::new(ret_ty),
            },
            size: 0,
        }
    }

    pub fn can_be_constant(&self) -> bool {
        match self.kind {
            TypeKind::BOOLEAN | TypeKind::INT64 | TypeKind::UINT64 => true,
            _ => false,
        }
    }

    pub fn size(&self, target: Target) -> usize {
        match self.kind {
            TypeKind::BOOLEAN => Self::boolean_size(target),
            TypeKind::CONSTSTR => Self::conststr_size(target),
            TypeKind::INT64 => Self::int64_size(target),
            TypeKind::UINT64 => Self::uint64_size(target),
            TypeKind::ENUM => 8,
            _ => unreachable!(),
        }
    }

    /// Int64型サイズ
    pub fn int64_size(target: Target) -> usize {
        match target {
            Target::X86_64 => 8,
            Target::AARCH64 => 8,
        }
    }

    /// Uint64型サイズ
    pub fn uint64_size(target: Target) -> usize {
        match target {
            Target::X86_64 => 8,
            Target::AARCH64 => 8,
        }
    }

    /// ポインタ型サイズ
    pub fn pointer_size(target: Target) -> usize {
        match target {
            Target::X86_64 => 8,
            Target::AARCH64 => 8,
        }
    }
    /// Boolean型サイズ
    pub fn boolean_size(target: Target) -> usize {
        match target {
            Target::X86_64 => 8,
            Target::AARCH64 => 8,
        }
    }
    /// ConstStr
    pub fn conststr_size(target: Target) -> usize {
        match target {
            Target::X86_64 => 8,
            Target::AARCH64 => 8,
        }
    }

    /// Int64型を新たに割り当てる
    pub fn new_int64(target: Target) -> Self {
        Self {
            kind: TypeKind::INT64,
            size: Self::int64_size(target),
        }
    }

    /// Uint64型を新たに割り当てる
    pub fn new_uint64(target: Target) -> Self {
        Self {
            kind: TypeKind::UINT64,
            size: Self::uint64_size(target),
        }
    }
    /// Boolean型を新たに割り当てる
    pub fn new_boolean(target: Target) -> Self {
        Self {
            kind: TypeKind::BOOLEAN,
            size: Self::boolean_size(target),
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
    pub fn new_const_str(target: Target) -> Self {
        Self {
            kind: TypeKind::CONSTSTR,
            size: Self::conststr_size(target),
        }
    }
    pub fn new_const(const_type: Type, expr: String, target: Target) -> Self {
        let size = const_type.size(target);
        Self {
            kind: TypeKind::CONST {
                const_type: Box::new(const_type),
                value: expr,
            },
            size,
        }
    }

    /// ポインタ型を新たに割り当てる
    pub fn new_pointer(to: Self, target: Target) -> Self {
        Self {
            kind: TypeKind::POINTER { to: Box::new(to) },
            size: Self::pointer_size(target),
        }
    }

    /// 構造体型型を新たに割り当てる
    pub fn new_struct(members: BTreeMap<String, (Box<Type>, usize)>, total_size: usize) -> Self {
        Self {
            kind: TypeKind::STRUCT { members },
            size: total_size,
        }
    }
    pub fn new_enum(size: usize) -> Self {
        Self {
            kind: TypeKind::ENUM,
            size,
        }
    }

    /// 構造体型であるか
    pub fn is_struct(&self) -> bool {
        match self.kind {
            TypeKind::STRUCT { members: _ } => true,
            _ => false,
        }
    }
    /// stantであるか
    pub fn is_constant(&self) -> bool {
        match self.kind {
            TypeKind::CONST {
                const_type: _,
                value: _,
            } => true,
            _ => false,
        }
    }
    /// 関数型であるか
    pub fn is_function(&self) -> bool {
        match &self.kind {
            TypeKind::FUNCTION { return_type: _ } => true,
            _ => false,
        }
    }
    /// ポインタ型であると解釈し, 指す型を取り出す
    pub fn pointer_to(&self) -> &Type {
        match &self.kind {
            TypeKind::POINTER { to } => to,
            _ => panic!("cannot call pointer_to() with not a pointer"),
        }
    }

    /// 定数であると解釈し,式文字列を取得する
    pub fn get_const_value(&self) -> String {
        match &self.kind {
            TypeKind::CONST {
                const_type: _,
                value,
            } => value.to_string(),
            _ => panic!("cannot call get_const_value() with not a constant"),
        }
    }
    pub fn get_const_type(&self) -> &Type {
        match &self.kind {
            TypeKind::CONST {
                const_type,
                value: _,
            } => const_type,
            _ => panic!("cannot call get_const_type() with not a constant"),
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
        members: BTreeMap<String, (Box<Type>, usize)>,
    },
    /// 関数型
    FUNCTION {
        return_type: Box<Type>,
        // args
    },
    /// 定数
    CONST {
        const_type: Box<Type>,
        value: String,
    },
    ENUM,
}
