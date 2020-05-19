use crate::compiler::resource as res;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
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

    pub fn set_type(&mut self, t: res::PType) {
        self.ptype = t;
    }
    pub fn get_type(&self) -> &res::PType {
        &self.ptype
    }

    pub fn type_size(&self) -> usize {
        self.ptype.type_size()
    }
    pub fn set_stack_offset(&mut self, offset: usize) {
        match self.kind {
            PVarKind::LOCAL(ref mut local_offset) => {
                *local_offset = offset;
            }
        }
    }
    pub fn get_stack_offset(&self) -> usize {
        match self.kind {
            PVarKind::LOCAL(offset) => offset,
        }
    }
}

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
#[allow(dead_code)]
pub enum PVarKind {
    LOCAL(usize), // stack offset
}
