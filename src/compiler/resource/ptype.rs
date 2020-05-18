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
    pub const GLOBAL_BOOLEAN_TYPE: Self = Self {
        kind: PTypeKind::BOOLEAN,
        size: 8,
    };
    pub const GLOBAL_NORETURN_TYPE: Self = Self {
        kind: PTypeKind::NORETURN,
        size: 0,
    };
    pub const GLOBAL_STR_TYPE: Self = Self {
        kind: PTypeKind::STR,
        size: 8,
    };
    pub const GLOBAL_UNRESOLVED_TYPE: Self = Self {
        kind: PTypeKind::UNRESOLVED(String::new()),
        size: 0,
    };

    fn new(k: PTypeKind, s: usize) -> Self {
        Self { kind: k, size: s }
    }

    pub fn new_int64() -> Self {
        Self::new(PTypeKind::INT64, 8)
    }
    pub fn new_noreturn() -> Self {
        Self::new(PTypeKind::NORETURN, 0)
    }
    pub fn new_str() -> Self {
        Self::new(PTypeKind::STR, 8)
    }
    pub fn new_unresolved(name: String) -> Self {
        Self::new(PTypeKind::UNRESOLVED(name), 0)
    }

    // TODO: サイズは1のほうが効率的
    pub fn new_boolean() -> Self {
        Self::new(PTypeKind::BOOLEAN, 8)
    }

    pub fn type_size(&self) -> usize {
        self.size
    }

    pub fn get_global_type_from(pt: &Self) -> Self {
        match &pt.kind {
            PTypeKind::BOOLEAN => Self::GLOBAL_BOOLEAN_TYPE,
            PTypeKind::INT64 => Self::GLOBAL_INT_TYPE,
            PTypeKind::STR => Self::GLOBAL_STR_TYPE,
            PTypeKind::NORETURN => Self::GLOBAL_NORETURN_TYPE,
            PTypeKind::UNRESOLVED(_name) => Self::GLOBAL_UNRESOLVED_TYPE,
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
    STR,
    NORETURN,
    BOOLEAN,
    UNRESOLVED(String),
}

impl PTypeKind {
    fn to_str(&self) -> &'static str {
        match self {
            Self::INT64 => "int64",
            Self::STR => "str",
            Self::NORETURN => "noreturn",
            Self::BOOLEAN => "boolean",
            Self::UNRESOLVED(_name) => "unresolved",
        }
    }
}
