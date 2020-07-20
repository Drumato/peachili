use crate::common::three_address_code::code_kind;
use id_arena::Id;

pub type CodeId = Id<Code>;

/// IRの最小単位
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Code {
    /// 種類
    pub kind: code_kind::CodeKind,

    // ベーシックブロック分割時に利用
    // pub label: Option<String>,
}

