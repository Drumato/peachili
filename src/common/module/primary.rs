use crate::common::module;

pub struct PrimaryModule {
    file_path: String,
    visited: bool,
    // requires: Vec<module::ExternalModule>,
}

impl PrimaryModule {
    pub fn new(fp: String) -> Self {
        Self {
            file_path: fp,
            visited: false,
            // requires: Vec::new(),
        }
    }
}

impl module::Module for PrimaryModule {
    fn mod_type(&self) -> module::ModuleType {
        module::ModuleType::PRIMARY
    }
    fn is_visited(&self) -> bool {
        self.visited
    }
    fn mod_path(&self) -> &String {
        &self.file_path
    }
    // fn mod_requires(&self) -> &Vec<module::Module> {
    // & self.requires
    // }
}
