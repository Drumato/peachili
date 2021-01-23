use std::collections::HashMap;

use crate::compiler::common::frontend::typed_ast;

pub struct Root {
    pub functions: Vec<typed_ast::Function>,
    pub constants: HashMap<String, typed_ast::Constant>,
}
