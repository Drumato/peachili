#[derive(Debug, Clone)]
pub enum PeachiliType {
    Int64,
    Uint64,
    ConstStr,
    Noreturn,
    Boolean,
}

impl PeachiliType {
    pub fn size(&self) -> usize {
        match self {
            PeachiliType::Int64 => 8,
            PeachiliType::Uint64 => 8,
            // とりあえず8
            PeachiliType::Boolean => 8,
            PeachiliType::ConstStr => 8,
            PeachiliType::Noreturn => 0,
        }
    }
}
