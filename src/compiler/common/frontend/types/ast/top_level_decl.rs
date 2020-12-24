use std::collections::HashMap;

use super::{ExprInfo, StmtInfo};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopLevelDecl<'a> {
    pub kind: TopLevelDeclKind<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TopLevelDeclKind<'a> {
    PubType {
        type_name: String,
        to: String,
    },
    PubConst {
        const_name: String,
        const_type: String,
        expr: ExprInfo<'a>,
    },
    Function {
        func_name: String,
        return_type: String,
        parameters: HashMap<String, String>,
        stmts: Vec<StmtInfo<'a>>,
    },
    Import {
        module_name: String,
    },
}
