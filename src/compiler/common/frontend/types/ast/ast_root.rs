use super::TopLevelDecl;

/// Root
#[derive(Debug, Clone)]
pub struct ASTRoot<'a> {
    pub decls: Vec<TopLevelDecl<'a>>,
}

impl<'a> Default for ASTRoot<'a> {
    fn default() -> Self {
        Self { decls: Vec::new() }
    }
}
