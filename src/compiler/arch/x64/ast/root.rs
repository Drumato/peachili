use std::collections::HashMap;

use crate::compiler::arch::x64;
pub struct Root {
    pub functions: Vec<x64::Function>,
    pub constants: HashMap<String, x64::Constant>,
}
