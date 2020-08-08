use std::collections::BTreeMap;

pub type StackFrame = BTreeMap<String, BTreeMap<String, FrameObject>>;

/// 型
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct FrameObject {
    /// 変数のオフセット
    /// 関数の場合は関数が割り当てるスタックフレームのサイズが入っている
    pub offset: usize,
}
