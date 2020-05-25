use std::collections::BTreeMap;

use crate::compiler::general::resource as res;

#[derive(PartialEq, Debug, Clone)]
pub struct TopLevelDecl {
    kind: TLDKind,
}

impl TopLevelDecl {
    fn new(kind: TLDKind) -> Self {
        Self { kind }
    }

    pub fn new_fn(fn_ret: res::PType, fn_args: Vec<(String, res::PType)>) -> Self {
        Self::new(TLDKind::FN(fn_ret, fn_args))
    }

    pub fn new_alias(src_type: res::PType) -> Self {
        Self::new(TLDKind::ALIAS(src_type))
    }

    pub fn get_return_type(&self) -> &res::PType {
        match &self.kind {
            TLDKind::FN(return_type, _arg_types) => return_type,
            _ => panic!("not a function"),
        }
    }
    pub fn get_args(&self) -> &[(String, res::PType)] {
        match &self.kind {
            TLDKind::FN(_return_type, arg_types) => arg_types,
            _ => panic!("not a function"),
        }
    }

    pub fn get_src_type(&self) -> &res::PType {
        match &self.kind {
            TLDKind::ALIAS(src_type) => src_type,
            _ => panic!("not an alias"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum TLDKind {
    // CONST,
    FN(res::PType, Vec<(String, res::PType)>),
    ALIAS(res::PType),
    // TYPE,
}

pub struct TLDResolver {
    tld_map: BTreeMap<String, TopLevelDecl>,
}

impl Default for TLDResolver {
    fn default() -> Self {
        Self {
            tld_map: BTreeMap::new(),
        }
    }
}

impl TLDResolver {
    pub fn insert_entry(&mut self, name: String, decl: TopLevelDecl) {
        assert!(self.tld_map.insert(name, decl).is_none());
    }
    pub fn give_map(self) -> BTreeMap<String, TopLevelDecl> {
        self.tld_map
    }
}
