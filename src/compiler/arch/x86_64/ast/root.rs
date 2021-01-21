use std::collections::HashMap;

use crate::compiler::arch::x86_64;
pub struct Root {
    pub functions: Vec<x86_64::Function>,
    pub constants: HashMap<String, x86_64::Constant>,
}
