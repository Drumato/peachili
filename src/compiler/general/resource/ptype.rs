use crate::compiler::general::resource as res;
use std::collections::BTreeMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PType {
    pub kind: PTypeKind,
    pub size: usize,
}

impl PType {
    // 型チェック時に毎回タイプを生成するとコストがかかる
    // ここではグローバルな実体の参照を取り回すことで，型検査を実装する
    pub const GLOBAL_INT_TYPE: Self = Self {
        kind: PTypeKind::INT64,
        size: 8,
    };
    pub const GLOBAL_UINT_TYPE: Self = Self {
        kind: PTypeKind::UINT64,
        size: 8,
    };
    pub const GLOBAL_BOOLEAN_TYPE: Self = Self {
        kind: PTypeKind::BOOLEAN,
        size: 8,
    };
    pub const GLOBAL_NORETURN_TYPE: Self = Self {
        kind: PTypeKind::NORETURN,
        size: 0,
    };
    pub const GLOBAL_CONSTSTR_TYPE: Self = Self {
        kind: PTypeKind::CONSTSTR,
        size: 8,
    };
    pub const GLOBAL_INVALID_TYPE: Self = Self {
        kind: PTypeKind::INVALID,
        size: 0,
    };

    fn new(k: PTypeKind, s: usize) -> Self {
        Self { kind: k, size: s }
    }

    pub fn new_int64() -> Self {
        Self::new(PTypeKind::INT64, 8)
    }
    pub fn new_uint64() -> Self {
        Self::new(PTypeKind::UINT64, 8)
    }
    pub fn new_noreturn() -> Self {
        Self::new(PTypeKind::NORETURN, 0)
    }
    pub fn new_conststr() -> Self {
        Self::new(PTypeKind::CONSTSTR, 8)
    }
    pub fn new_unresolved(name: res::IdentName) -> Self {
        Self::new(PTypeKind::UNRESOLVED(name), 0)
    }
    pub fn new_invalid() -> Self {
        Self::new(PTypeKind::INVALID, 0)
    }
    pub fn new_pointer(inner: PType, ref_local: bool) -> Self {
        Self::new(PTypeKind::POINTER(Box::new(inner), ref_local), 8)
    }
    pub fn new_struct(mut members: BTreeMap<res::PStringId, (PType, usize)>) -> Self {
        let mut total_size: usize = 0;
        // メンバのサイズを合計
        for (_name, (member_t, offset)) in members.iter_mut() {
            *offset = total_size;
            total_size += member_t.type_size();
        }
        Self::new(PTypeKind::STRUCT { members }, total_size)
    }

    // TODO: サイズは1のほうが効率的
    pub fn new_boolean() -> Self {
        Self::new(PTypeKind::BOOLEAN, 8)
    }

    pub fn is_unsigned(&self) -> bool {
        match &self.kind {
            PTypeKind::UINT64 => true,
            _ => false,
        }
    }

    pub fn is_pointer(&self) -> bool {
        match &self.kind {
            PTypeKind::POINTER(_inner, _ref_local) => true,
            _ => false,
        }
    }
    pub fn is_struct(&self) -> bool {
        match &self.kind {
            PTypeKind::STRUCT { members: _ } => true,
            _ => false,
        }
    }

    pub fn dereference(&self) -> Self {
        match &self.kind {
            PTypeKind::POINTER(inner, _ref_local) => *inner.clone(),
            _ => panic!("not a pointer"),
        }
    }
    pub fn ref_local(&self) -> bool {
        match &self.kind {
            PTypeKind::POINTER(_inner, ref_local) => *ref_local,
            _ => panic!("not a pointer"),
        }
    }
    pub fn copy_members(&self) -> BTreeMap<res::PStringId, (PType, usize)> {
        match &self.kind {
            PTypeKind::STRUCT { members } => members.clone(),
            _ => panic!("not a struct"),
        }
    }

    pub fn get_members(&self) -> &BTreeMap<res::PStringId, (PType, usize)> {
        match &self.kind {
            PTypeKind::STRUCT { members } => &members,
            _ => panic!("not a struct"),
        }
    }

    pub fn type_size(&self) -> usize {
        self.size
    }

    pub fn get_global_type_from(pt: &Self) -> Self {
        match &pt.kind {
            PTypeKind::BOOLEAN => Self::GLOBAL_BOOLEAN_TYPE,
            PTypeKind::INT64 => Self::GLOBAL_INT_TYPE,
            PTypeKind::UINT64 => Self::GLOBAL_UINT_TYPE,
            PTypeKind::CONSTSTR => Self::GLOBAL_CONSTSTR_TYPE,
            PTypeKind::NORETURN => Self::GLOBAL_NORETURN_TYPE,
            PTypeKind::STRUCT { members: _ } => {
                panic!("unexpected calling get_global_type_from with struct type")
            }
            PTypeKind::UNRESOLVED(_name) => {
                panic!("unexpected calling get_global_type_from with unresolved type")
            }
            PTypeKind::POINTER(_inner, _ref_local) => {
                panic!("unexpected calling get_global_type_from with pointer type")
            }
            PTypeKind::INVALID => Self::GLOBAL_INVALID_TYPE,
        }
    }
}

impl std::fmt::Display for PType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind.to_str())
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PTypeKind {
    INT64,
    UINT64,
    CONSTSTR,
    NORETURN,
    BOOLEAN,
    UNRESOLVED(res::IdentName),

    /// inner_type, is_local
    POINTER(Box<PType>, bool),

    /// member_name -> (member_type, member_relative_offset)
    STRUCT {
        members: BTreeMap<res::PStringId, (PType, usize)>,
    },
    INVALID,
}

impl PTypeKind {
    fn to_str(&self) -> &'static str {
        match self {
            Self::INT64 => "Int64",
            Self::UINT64 => "Uint64",
            Self::CONSTSTR => "ConstStr",
            Self::NORETURN => "Noreturn",
            Self::BOOLEAN => "Boolean",
            Self::UNRESOLVED(_name) => "Unresolved",
            Self::POINTER(_inner, _ref_local) => "Pointer",
            Self::STRUCT { members: _ } => "struct",
            Self::INVALID => "Invalid",
        }
    }
}
