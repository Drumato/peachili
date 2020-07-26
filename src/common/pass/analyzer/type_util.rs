use std::collections::BTreeMap;
use crate::common::{option, tld};
use crate::common::peachili_type::Type;

/// 型のサイズの計算
pub fn resolve_type_size(
    _tld_map: &BTreeMap<String, tld::TopLevelDecl>,
    t: ForCalcTypeSize,
    target: option::Target,
) -> usize {
    match t {
        ForCalcTypeSize::INT64 => Type::int64_size(target),
        ForCalcTypeSize::UINT64 => Type::uint64_size(target),
        ForCalcTypeSize::POINTER => Type::pointer_size(target),
        ForCalcTypeSize::BOOLEAN => Type::boolean_size(target),
        ForCalcTypeSize::CONSTSTR => Type::conststr_size(target),
    }
}

/// アーキテクチャごとに型のサイズを割り当てるためだけに使用．
pub enum ForCalcTypeSize {
    INT64,
    UINT64,
    POINTER,
    BOOLEAN,
    CONSTSTR,
}