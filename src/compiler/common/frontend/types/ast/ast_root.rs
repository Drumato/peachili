use super::TopLevelDecl;

/// Root
#[derive(Debug, Clone)]
pub struct ASTRoot {
    pub decls: Vec<TopLevelDecl>,
    pub module_name: String,
}

impl Default for ASTRoot {
    fn default() -> Self {
        Self {
            decls: Vec::new(),
            module_name: String::new(),
        }
    }
}
