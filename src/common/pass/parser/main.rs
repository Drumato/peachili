use crate::common::ast;

use crate::common::pass::parser::context::Context;

pub fn main(file_contents: String, module_name: String) -> ast::ASTRoot {
    let mut ast_root: ast::ASTRoot = Default::default();
    let mut ctxt: Context = Default::default();
    ctxt.module_name = module_name;

    // program -> toplevel*
    ast_root
}

#[cfg(test)]
mod toplevel_tests {
    use super::*;

    #[test]
    fn type_alias_test() {}

    #[test]
    fn struct_def_test() {}

    #[test]
    fn member_block_test() {}

    #[test]
    fn arg_list_test() {}

    #[test]
    fn func_def_test() {}

    #[test]
    fn main_test() {}
}
