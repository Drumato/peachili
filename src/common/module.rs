use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};

use id_arena::*;

use crate::compiler::general::resource as res;

#[derive(Clone)]
pub struct ModuleVec {
    allocator: ModuleAllocator,
    modules: Arc<Mutex<Vec<ModuleId>>>,
}

impl ModuleVec {
    pub fn get_locked_modules(&self) -> MutexGuard<Vec<ModuleId>> {
        self.modules.lock().unwrap()
    }
}

impl Default for ModuleVec {
    fn default() -> Self {
        Self {
            allocator: Default::default(),
            modules: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Clone)]
pub struct ModuleAllocator {
    arena: Arena<Module>,
}

impl Default for ModuleAllocator {
    fn default() -> Self {
        Self {
            arena: Arena::new(),
        }
    }
}

impl ModuleAllocator {
    pub fn alloc_main_module(&mut self, fp: String) -> ModuleId {
        self.arena.alloc(Module::new_primary(fp))
    }
    pub fn alloc_external_module(&mut self, fp: String) -> ModuleId {
        self.arena.alloc(Module::new_external(fp))
    }
    pub fn get_module_as_mut(&mut self, id: ModuleId) -> Option<&mut Module> {
        self.arena.get_mut(id)
    }
    pub fn get_module_ref(&self, id: &ModuleId) -> Option<&Module> {
        self.arena.get(*id)
    }
}

pub type ModuleId = Id<Module>;

#[derive(Clone)]
pub struct Module {
    pub kind: ModuleKind,
    pub visited: bool,
    pub file_path: String,
    pub requires: ModuleVec,
    pub subs: ModuleVec,
    pub functions: Vec<res::PFunction>,
    pub tokens: Vec<res::Token>,
    pub const_arena: Arc<Mutex<res::ConstAllocator>>,
    pub hash_value: u64,
}

#[allow(dead_code)]
impl Module {
    fn new(kind: ModuleKind, file_path: String) -> Self {
        let mut hasher = DefaultHasher::new();

        // 前段のprimary() で文字列リテラルであることを検査しているのでunwrap()してよい．
        file_path.hash(&mut hasher);

        Self {
            kind,
            visited: false,
            file_path,
            requires: Default::default(),
            subs: Default::default(),
            functions: Vec::new(),
            tokens: Vec::new(),
            const_arena: Arc::new(Mutex::new(Default::default())),
            hash_value: hasher.finish(),
        }
    }

    pub fn new_primary(file_path: String) -> Self {
        Self::new(ModuleKind::PRIMARY, file_path)
    }

    pub fn new_external(file_path: String) -> Self {
        Self::new(ModuleKind::EXTERNAL, file_path)
    }

    /// subライブラリを保持しているか
    pub fn is_parent(&self) -> bool {
        !self.get_locked_submodules().is_empty()
    }

    pub fn set_tokens(&mut self, tokens: Vec<res::Token>) {
        self.tokens = tokens;
    }
    pub fn set_const_arena(&mut self, allocator: Arc<Mutex<res::ConstAllocator>>) {
        self.const_arena = allocator;
    }

    pub fn get_const_pool_ref(&self) -> MutexGuard<res::ConstAllocator> {
        self.const_arena.lock().unwrap()
    }
    pub fn get_tokens_as_mut(&mut self) -> &mut Vec<res::Token> {
        &mut self.tokens
    }

    pub fn get_locked_requires(&self) -> MutexGuard<Vec<ModuleId>> {
        self.requires.get_locked_modules()
    }
    pub fn get_locked_submodules(&self) -> MutexGuard<Vec<ModuleId>> {
        self.subs.get_locked_modules()
    }
    pub fn set_file_path(&mut self, fp: String) {
        self.file_path = fp;
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum ModuleKind {
    PRIMARY,
    EXTERNAL,
}
