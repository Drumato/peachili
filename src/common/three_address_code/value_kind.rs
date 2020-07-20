/// Valueの種類
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum ValueKind {
    INTLITERAL {
        value: i64,
    },
    UINTLITERAL {
        value: u64,
    },
    TEMP {
        number: usize,
    },
    ID {
        name: String,
    },
}