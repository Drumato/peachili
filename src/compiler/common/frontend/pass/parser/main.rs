use super::{primitive, Parser};
use crate::compiler::common::frontend::ast;
use nom::{branch::alt, bytes::complete::take_while1, multi::many0, sequence::delimited, IResult};
/// program -> toplevel*
pub fn main<'a>(
    parser: &'a Parser<'a>,
    _module_name: &'a str,
    file_contents: &'a str,
) -> Result<ast::ASTRoot<'a>, Box<dyn std::error::Error + 'a>> {
    let mut ast_root: ast::ASTRoot = Default::default();

    let (_rest, top_level_decls) = many0(alt((
        parser.import_directive(),
        parser.function_declaration(),
    )))(file_contents)?;

    ast_root.decls = top_level_decls;
    Ok(ast_root)
}

impl<'a> Parser<'a> {
    fn import_directive(&'a self) -> impl Fn(&str) -> IResult<&str, ast::TopLevelDecl<'a>> {
        move |i: &str| {
            let (rest, _) = primitive::keyword("import")(i)?;
            let (rest, module_name) = take_while1(|b| b != ';')(rest)?;
            let (rest, _) = primitive::symbol(";")(rest)?;

            Ok((
                rest,
                ast::TopLevelDecl {
                    kind: ast::TopLevelDeclKind::Import {
                        module_name: module_name.to_string(),
                    },
                },
            ))
        }
    }

    fn function_declaration(&'a self) -> impl Fn(&'a str) -> IResult<&str, ast::TopLevelDecl<'a>> {
        move |i: &str| {
            let (rest, _) = primitive::keyword("func")(i)?;
            let (rest, func_name) = self.identifier_string()(rest)?;

            let (rest, _) = primitive::symbol("(")(rest)?;
            let (rest, _) = primitive::symbol(")")(rest)?;

            let (rest, return_type) = self.identifier_string()(rest)?;

            let (rest, stmts) = delimited(
                primitive::symbol("{"),
                many0(self.statement()),
                primitive::symbol("}"),
            )(rest)?;
            Ok((
                rest,
                ast::TopLevelDecl {
                    kind: ast::TopLevelDeclKind::Function {
                        func_name,
                        return_type,
                        stmts,
                    },
                },
            ))
        }
    }
}

#[cfg(test)]
mod toplevel_tests {
    use super::*;

    #[test]
    fn parser_main_test() {
        let arena = Default::default();
        let parser = Parser::new(&arena);

        let result = main(&parser, "main", "import x64;\n\nfunc main() Noreturn {}");
        assert!(result.is_ok());

        let ast_root = result.unwrap();
        assert_eq!(2, ast_root.decls.len());
    }

    #[test]
    fn import_directive_test() {
        let arena = Default::default();
        let parser = Parser::new(&arena);

        let result = parser.import_directive()("import x64;");
        assert!(result.is_ok());

        let (rest, import_decl) = result.unwrap();
        assert_eq!("", rest);
        assert_eq!(
            ast::TopLevelDeclKind::Import {
                module_name: "x64".to_string()
            },
            import_decl.kind
        );
    }

    #[test]
    fn function_declaration_test() {
        let arena = Default::default();
        let parser = Parser::new(&arena);

        let result = parser.function_declaration()("func main() Noreturn {}");
        assert!(result.is_ok());

        let (rest, f) = result.unwrap();
        assert_eq!("", rest);

        assert_eq!(
            ast::TopLevelDeclKind::Function {
                func_name: "main".to_string(),
                return_type: "Noreturn".to_string(),
                stmts: Vec::new(),
            },
            f.kind
        );
    }
}
