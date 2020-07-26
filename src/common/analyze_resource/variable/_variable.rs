use crate::common::peachili_type::*;

/// Peachiliの型エントリ
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Variable{
    /// 変数の型
    pt: Type,
}