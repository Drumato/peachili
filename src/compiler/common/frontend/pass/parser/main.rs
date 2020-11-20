use crate::compiler::common::frontend::{allocator, ast, parse_context};

pub fn main<'a>(
    arena: &allocator::Allocator<'a>,
    file_contents: String,
    module_name: String,
) -> ast::ASTRoot {
    let mut ast_root: ast::ASTRoot = Default::default();
    let mut ctxt: parse_context::Context = Default::default();
    ctxt.module_name = module_name;

    // program -> toplevel*
    ast_root
}

#[cfg(test)]
mod toplevel_tests {
    use super::*;
    #[test]
    fn func_def_test() {}
}
