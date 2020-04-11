pub struct Immediate {
    kind: ImmediateKind,
}

impl Immediate {
    pub fn to_at(&self) -> String {
        format!("${}", self.kind)
    }
    pub fn new_int64(v: i64) -> Self {
        Self::new(ImmediateKind::INT64(v))
    }
    pub fn new_uint64(v: u64) -> Self {
        Self::new(ImmediateKind::UINT64(v))
    }

    fn new(ik: ImmediateKind) -> Self {
        Self { kind: ik }
    }
}

pub enum ImmediateKind {
    INT64(i64),
    UINT64(u64),
}

impl std::fmt::Display for ImmediateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::INT64(v64) => write!(f, "{}", v64),
            Self::UINT64(v64) => write!(f, "{}", v64),
        }
    }
}
