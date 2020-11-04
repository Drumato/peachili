use std::collections::HashSet;

/// パース処理に必要な情報を集約する構造体
pub struct Context {
    pub called_functions: HashSet<String>,
    pub module_name: String,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            called_functions: HashSet::new(),
            module_name: String::new(),
        }
    }
}
