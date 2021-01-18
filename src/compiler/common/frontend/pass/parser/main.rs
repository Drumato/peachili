use super::*;
use crate::compiler::common::frontend::ast;
use nom::{
    branch::alt,
    bytes::complete::take_while1,
    multi::many0,
    sequence::{delimited, tuple},
    IResult,
};
use primitive::{identifier_string, list_structure, symbol};
use std::collections::HashMap;

/// program -> toplevel*
pub fn main<'a>(
    module_name: &'a str,
    file_contents: &'a str,
) -> Result<ast::ASTRoot, Box<dyn std::error::Error + 'a>> {
    let mut ast_root: ast::ASTRoot = Default::default();
    ast_root.module_name = module_name.to_string();

    let (_rest, top_level_decls) = many0(alt((
        pubtype_declaration(module_name),
        import_directive(),
        pubconst_declaration(module_name),
        function_declaration(module_name),
    )))(file_contents)?;

    ast_root.decls = top_level_decls;
    Ok(ast_root)
}
fn import_directive<'a>() -> impl Fn(&str) -> IResult<&str, ast::TopLevelDecl> {
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

fn pubtype_declaration<'a>(
    module_name: &'a str,
) -> impl Fn(&'a str) -> IResult<&str, ast::TopLevelDecl> {
    move |i: &str| {
        let (rest, _) = primitive::keyword("pubtype")(i)?;
        let (rest, type_name) = primitive::identifier_string()(rest)?;

        let (rest, _) = symbol("=")(rest)?;
        let (rest, to) = primitive::identifier_string()(rest)?;
        let (rest, _) = symbol(";")(rest)?;

        Ok((
            rest,
            ast::TopLevelDecl {
                kind: ast::TopLevelDeclKind::PubType {
                    type_name: format!("{}::{}", module_name, type_name),
                    to,
                },
            },
        ))
    }
}

fn pubconst_declaration<'a>(
    module_name: &'a str,
) -> impl Fn(&'a str) -> IResult<&str, ast::TopLevelDecl> {
    move |i: &str| {
        let (rest, _) = primitive::keyword("pubconst")(i)?;
        let (rest, const_name) = primitive::identifier_string()(rest)?;

        let (rest, _) = symbol(":")(rest)?;
        let (rest, const_type) = primitive::identifier_string()(rest)?;

        let (rest, _) = symbol("=")(rest)?;
        let (rest, expr) = expression()(rest)?;
        let (rest, _) = symbol(";")(rest)?;

        Ok((
            rest,
            ast::TopLevelDecl {
                kind: ast::TopLevelDeclKind::PubConst {
                    const_name: format!("{}::{}", module_name, const_name),
                    const_type,
                    expr,
                },
            },
        ))
    }
}

fn function_declaration<'a>(
    module_name: &'a str,
) -> impl Fn(&'a str) -> IResult<&str, ast::TopLevelDecl> {
    move |i: &str| {
        let (rest, _) = primitive::keyword("func")(i)?;
        let (rest, func_name) = primitive::identifier_string()(rest)?;

        let mut parameters = HashMap::new();

        let (rest, params) = list_structure(primitive::Delimiter::Paren, ",", move |i: &str| {
            tuple((identifier_string(), identifier_string()))(i)
        })(rest)?;
        for (name, ty) in params.iter() {
            parameters.insert(name.to_string(), ty.to_string());
        }

        let (rest, return_type) = primitive::identifier_string()(rest)?;

        let (rest, stmts) = delimited(
            primitive::symbol("{"),
            many0(statement()),
            primitive::symbol("}"),
        )(rest)?;
        Ok((
            rest,
            ast::TopLevelDecl {
                kind: ast::TopLevelDeclKind::Function {
                    func_name: format!("{}::{}", module_name, func_name),
                    return_type,
                    parameters,
                    stmts,
                },
            },
        ))
    }
}

#[cfg(test)]
mod toplevel_tests {
    use ast::Expr;

    use super::*;

    #[test]
    fn parser_main_test() {
        let result = main("main", "import x64;\n\nfunc main() Noreturn {}");
        assert!(result.is_ok());

        let ast_root = result.unwrap();
        assert_eq!(2, ast_root.decls.len());
    }

    #[test]
    fn import_directive_test() {
        let result = import_directive()("import x64;");
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
        let result =
            function_declaration("main")("func main(a Int64, b Int64, c Int64) Noreturn {}");
        assert!(result.is_ok());

        let (rest, f) = result.unwrap();
        assert_eq!("", rest);

        assert_eq!(
            ast::TopLevelDeclKind::Function {
                func_name: "main::main".to_string(),
                return_type: "Noreturn".to_string(),
                parameters: {
                    let mut h = HashMap::new();
                    h.insert("a".to_string(), "Int64".to_string());
                    h.insert("b".to_string(), "Int64".to_string());
                    h.insert("c".to_string(), "Int64".to_string());
                    h
                },
                stmts: Vec::new(),
            },
            f.kind
        );
    }

    #[test]
    fn pubtype_declaration_test() {
        let result = pubtype_declaration("main")("pubtype FileDescriptor = Uint64;");
        assert!(result.is_ok());

        let (r, pubtype) = result.unwrap();
        assert_eq!("", r);

        assert_eq!(
            ast::TopLevelDeclKind::PubType {
                type_name: "main::FileDescriptor".to_string(),
                to: "Uint64".to_string(),
            },
            pubtype.kind,
        );
    }

    #[test]
    fn pubconst_declaration_test() {
        let result = pubconst_declaration("main")("pubconst STDIN : FileDescriptor = u0;");

        assert!(result.is_ok());

        let (r, pubconst) = result.unwrap();
        assert_eq!("", r);

        assert_eq!(
            ast::TopLevelDeclKind::PubConst {
                const_name: "main::STDIN".to_string(),
                const_type: "FileDescriptor".to_string(),
                expr: Expr {
                    kind: ast::ExprKind::UnsignedInteger { value: 0 }
                },
            },
            pubconst.kind,
        );
    }
}
