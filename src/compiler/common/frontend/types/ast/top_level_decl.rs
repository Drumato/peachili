use super::StmtInfo;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TopLevelDecl<'a> {
    pub kind: TopLevelDeclKind<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TopLevelDeclKind<'a> {
    Function {
        func_name: String,
        return_type: String,
        stmts: Vec<StmtInfo<'a>>,
    },
    Import {
        module_name: String,
    },
}
