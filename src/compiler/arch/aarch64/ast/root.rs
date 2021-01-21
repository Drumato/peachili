use std::collections::HashMap;

use crate::compiler::arch::aarch64;
pub struct Root {
    pub functions: Vec<aarch64::Function>,
    pub constants: HashMap<String, aarch64::Constant>,
}
