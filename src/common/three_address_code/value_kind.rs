/// Valueの種類
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum ValueKind {
    INTLITERAL { value: i64 },
    UINTLITERAL { value: u64 },
    TEMP { number: usize },
    ID { name: String },
    BOOLEANLITERAL { truth: bool },
    STRINGLITERAL { contents: String },
}

impl ValueKind {
    pub fn dump(&self) -> String {
        match self {
            ValueKind::INTLITERAL { value } => value.to_string(),
            ValueKind::UINTLITERAL { value } => value.to_string(),
            ValueKind::BOOLEANLITERAL { truth } => truth.to_string(),
            ValueKind::STRINGLITERAL { contents } => contents.to_string(),

            ValueKind::TEMP { number } => format!("temp{}", number),
            ValueKind::ID { name } => name.to_string(),
        }
    }
}
