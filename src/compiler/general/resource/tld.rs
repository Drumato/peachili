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

    pub fn new_fn(fn_ret: res::PType, fn_args: Vec<(res::PStringId, res::PType)>) -> Self {
        Self::new(TLDKind::FN(fn_ret, fn_args))
    }

    pub fn new_alias(src_type: res::PType) -> Self {
        Self::new(TLDKind::ALIAS(src_type))
    }
    pub fn new_struct(members: BTreeMap<res::PStringId, (res::PType, usize)>) -> Self {
        Self::new(TLDKind::STRUCT(members))
    }

    pub fn to_ptype(&self) -> res::PType {
        match &self.kind {
            TLDKind::ALIAS(src_type) => src_type.clone(),
            TLDKind::STRUCT(ms) => {
                let mut members = BTreeMap::new();

                for (m_name, (m_t, m_off)) in ms.iter() {
                    members.insert(*m_name, (m_t.clone(), *m_off));
                }

                res::PType::new_struct(members)
            }
            _ => unimplemented!(),
        }
    }

    pub fn get_return_type(&self) -> &res::PType {
        match &self.kind {
            TLDKind::FN(return_type, _arg_types) => return_type,
            _ => panic!("not a function"),
        }
    }
    pub fn get_args(&self) -> &[(res::PStringId, res::PType)] {
        match &self.kind {
            TLDKind::FN(_return_type, arg_types) => arg_types,
            _ => panic!("not a function"),
        }
    }

    pub fn get_members(&self) -> &BTreeMap<res::PStringId, (res::PType, usize)> {
        match &self.kind {
            TLDKind::STRUCT(members) => members,
            _ => panic!("not a struct"),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum TLDKind {
    // CONST,
    FN(res::PType, Vec<(res::PStringId, res::PType)>),
    ALIAS(res::PType),
    STRUCT(BTreeMap<res::PStringId, (res::PType, usize)>),
}

pub struct TLDResolver {
    tld_map: BTreeMap<res::PStringId, TopLevelDecl>,
}

impl Default for TLDResolver {
    fn default() -> Self {
        Self {
            tld_map: BTreeMap::new(),
        }
    }
}

impl TLDResolver {
    pub fn insert_entry(&mut self, name_id: res::PStringId, decl: TopLevelDecl) {
        assert!(self.tld_map.insert(name_id, decl).is_none());
    }
    pub fn give_map(self) -> BTreeMap<res::PStringId, TopLevelDecl> {
        self.tld_map
    }
}
