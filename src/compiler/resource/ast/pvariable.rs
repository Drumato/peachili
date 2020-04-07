use crate::compiler::resource as res;

#[derive(Clone)]
#[allow(dead_code)]
pub struct PVariable {
    kind: PVarKind,
    ptype: res::PType,
}

impl PVariable {
    fn new(var_kind: PVarKind, var_type: res::PType) -> Self {
        Self {
            kind: var_kind,
            ptype: var_type,
        }
    }

    pub fn new_local(var_type: res::PType) -> Self {
        Self::new(PVarKind::LOCAL(0), var_type)
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum PVarKind {
    LOCAL(usize), // stack offset
}
