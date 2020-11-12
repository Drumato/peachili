/// Root
#[derive(Debug, Clone)]
pub struct ASTRoot {}

impl Default for ASTRoot {
    fn default() -> Self {
        Self {}
    }
}

impl ASTRoot {
    /// 別モジュールのASTRootを吸収する
    pub fn absorb(&mut self, _target: Self) {}
}
