pub struct Module {
    pub kind: ModuleKind,
    pub visited: bool,
    pub file_path: String,
    pub requires: Vec<Module>,
}

#[allow(dead_code)]
impl Module {
    fn new(kind: ModuleKind, file_path: String) -> Self {
        Self {
            kind,
            visited: false,
            file_path,
            requires: Vec::new(),
        }
    }
    pub fn new_primary(file_path: String) -> Self {
        Self::new(ModuleKind::PRIMARY, file_path)
    }
    pub fn new_external(file_path: String) -> Self {
        Self::new(ModuleKind::EXTERNAL, file_path)
    }
}

#[allow(dead_code)]
pub enum ModuleKind {
    PRIMARY,
    EXTERNAL,
}
