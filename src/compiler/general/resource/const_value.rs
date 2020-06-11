use id_arena::{Arena, Id};

use crate::compiler::general::resource as res;

#[derive(PartialEq, Debug, Clone)]
pub struct ConstAllocator {
    string_pool: Arena<PString>,
}

impl Default for ConstAllocator {
    fn default() -> Self {
        Self {
            string_pool: Arena::new(),
        }
    }
}

impl ConstAllocator {
    pub fn alloc_string(&mut self, v: String) -> PStringId {
        self.string_pool.alloc(PString::new(v))
    }

    pub fn get(&self, id: res::PStringId) -> Option<&res::PString> {
        self.string_pool.get(id)
    }
}

pub type PStringId = Id<PString>;

#[derive(PartialEq, Debug, Clone)]
pub struct PString {
    value: String,
}

impl Default for PString {
    fn default() -> Self {
        Self {
            value: String::new(),
        }
    }
}

impl std::fmt::Display for PString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl PString {
    pub fn new(v: String) -> Self {
        Self { value: v }
    }
    pub fn copy_value(&self) -> String {
        self.value.to_string()
    }
    pub fn len(&self) -> usize {
        self.value.len()
    }
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn compare_str(&self, s: String) -> bool {
        self.value == s
    }
}
