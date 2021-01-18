use std::collections::HashMap;

use super::{Expr, Stmt};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopLevelDecl {
    pub kind: TopLevelDeclKind,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TopLevelDeclKind {
    PubType {
        type_name: String,
        to: String,
    },
    PubConst {
        const_name: String,
        const_type: String,
        expr: Expr,
    },
    Function {
        func_name: String,
        return_type: String,
        parameters: HashMap<String, String>,
        stmts: Vec<Stmt>,
    },
    Import {
        module_name: String,
    },
}
