#[allow(dead_code)]
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PType {
    pub kind: PTypeKind,
    pub size: usize,
}

impl PType {
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

    // TODO: サイズは1のほうが効率的
    pub fn new_boolean() -> Self {
        Self::new(PTypeKind::BOOLEAN, 8)
    }

    pub fn type_size(&self) -> usize {
        self.size
    }
}

impl std::fmt::Display for PType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind.to_str())
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PTypeKind {
    INT64,
    STR,
    NORETURN,
    BOOLEAN,
}

impl PTypeKind {
    fn to_str(&self) -> &'static str {
        match self {
            Self::INT64 => "int64",
            Self::STR => "str",
            Self::NORETURN => "noreturn",
            Self::BOOLEAN => "boolean",
        }
    }
}
