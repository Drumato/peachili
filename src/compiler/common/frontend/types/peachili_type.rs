#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PeachiliType {
    pub kind: PTKind,
    pub size: usize,
}

impl PeachiliType {
    pub fn new(k: PTKind, s: usize) -> Self {
        Self { kind: k, size: s }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum PTKind {
    Int64,
    Uint64,
    ConstStr,
    Noreturn,
    Boolean,
}
