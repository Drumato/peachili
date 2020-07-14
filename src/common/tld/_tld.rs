use crate::common::tld::tld_kind;

/// 宣言
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct TopLevelDecl {
    pub kind: tld_kind::TLDKind,
}


impl TopLevelDecl {
    pub fn new(k: tld_kind::TLDKind) -> Self {
        Self {
            kind: k,
        }
    }
}