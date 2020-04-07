#[allow(dead_code)]
#[derive(Clone)]
pub struct PType {
    kind: PTypeKind,
    size: usize,
}

impl PType {
    fn new(k: PTypeKind, s: usize) -> Self {
        Self { kind: k, size: s }
    }

    pub fn new_int64() -> Self {
        Self::new(PTypeKind::INT64, 8)
    }
}

impl std::fmt::Display for PType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind.to_str())
    }
}

#[derive(Clone)]
pub enum PTypeKind {
    INT64,
}

impl PTypeKind {
    fn to_str(&self) -> &'static str {
        match self {
            Self::INT64 => "int64",
        }
    }
}
