use std::cell::RefCell;

use crate::compiler::resource as res;

#[derive(Clone)]
pub struct Module<'a> {
    pub kind: ModuleKind,
    pub visited: bool,
    pub file_path: String,
    pub requires: RefCell<Vec<Mod<'a>>>,
    pub subs: RefCell<Vec<Mod<'a>>>,
    pub functions: Vec<res::PFunction>,
}

type Mod<'a> = &'a Module<'a>;

#[allow(dead_code)]
impl<'a> Module<'a> {
    fn new(kind: ModuleKind, file_path: String) -> Self {
        Self {
            kind,
            visited: false,
            file_path,
            requires: RefCell::new(Vec::new()),
            subs: RefCell::new(Vec::new()),
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

impl<'a> std::fmt::Display for Module<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} - {}\n", self.file_path, self.kind.to_str())?;

        write!(f, "subs:\n")?;

        for sub in self.subs.borrow().iter() {
            write!(f, "\t{}\n", sub.file_path)?;
        }

        write!(f, "requires:\n")?;

        for req in self.requires.borrow().iter() {
            write!(f, "\t{}\n", req.file_path)?;
        }

        Ok(())
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum ModuleKind {
    PRIMARY,
    EXTERNAL,
}

impl ModuleKind {
    fn to_str(&self) -> &'static str {
        match self {
            Self::PRIMARY => "primary",
            Self::EXTERNAL => "external",
        }
    }
}
