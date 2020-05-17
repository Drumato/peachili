use std::collections::BTreeMap;

use crate::compiler::resource as res;

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

    pub fn get_return_type(&self) -> &res::PType {
        match &self.kind {
            TLDKind::FN(return_type, _arg_types) => return_type,
            // _ => panic!("not a function"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum TLDKind {
    // CONST,
    FN(res::PType, Vec<(String, res::PType)>),
    // ALIAS,
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
        self.tld_map.insert(name, decl);
    }
    pub fn give_map(self) -> BTreeMap<String, TopLevelDecl> {
        self.tld_map
    }
}
