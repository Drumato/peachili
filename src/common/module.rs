use crate::common::position;

pub trait Functionable {
    fn get_name(&self) -> &String;
    fn def_position(&self) -> &position::Position;
    // fn return_type(&self) -> PType;
}

pub struct Module<T: Functionable> {
    pub kind: ModuleKind,
    pub visited: bool,
    pub file_path: String,
    pub requires: Vec<Module<T>>,
    pub functions: Vec<T>,
}

#[allow(dead_code)]
impl<T: Functionable> Module<T> {
    fn new(kind: ModuleKind, file_path: String) -> Self {
        Self {
            kind,
            visited: false,
            file_path,
            requires: Vec::new(),
            functions: Vec::new(),
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
