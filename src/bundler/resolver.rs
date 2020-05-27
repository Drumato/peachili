use crate::common::module;

pub struct Resolver<'a> {
    allocator: &'a mut module::ModuleAllocator,
}

impl<'a> Resolver<'a> {
    pub fn new(alloc: &'a mut module::ModuleAllocator) -> Self {
        Self { allocator: alloc }
    }

    pub fn alloc_main_module(&mut self, fp: String) -> module::ModuleId {
        self.allocator.alloc_main_module(fp)
    }

    pub fn alloc_external_module(&mut self, fp: String) -> module::ModuleId {
        self.allocator.alloc_external_module(fp)
    }

    pub fn get_module_as_mut(&mut self, id: module::ModuleId) -> Option<&mut module::Module> {
        self.allocator.get_module_as_mut(id)
    }

    pub fn set_visited_to_given_id(&mut self, id: module::ModuleId, visited: bool) {
        self.allocator.get_module_as_mut(id).unwrap().visited = visited;
    }
}
