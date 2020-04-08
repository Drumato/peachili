use crate::compiler::resource as res;

#[derive(Clone)]
pub struct Module {
    pub kind: ModuleKind,
    pub visited: bool,
    pub file_path: String,
    pub requires: Vec<Module>,
    pub subs: Vec<Module>,
    pub functions: Vec<res::PFunction>,
}

#[allow(dead_code)]
impl Module {
    fn new(kind: ModuleKind, file_path: String) -> Self {
        Self {
            kind,
            visited: false,
            file_path,
            requires: Vec::new(),
            subs: Vec::new(),
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

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} - {}\n", self.file_path, self.kind.to_str())?;

        write!(f, "subs:\n")?;

        for sub in self.subs.iter() {
            write!(f, "\t{}\n", sub.file_path)?;
        }

        write!(f, "requires:\n")?;

        for req in self.requires.iter() {
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
