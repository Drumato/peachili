use crate::compiler::common::frontend::{
    pass::parser::{primitive, Parser},
    types::ast,
};

use nom::IResult;

type IResultStmt<'a> = IResult<&'a str, ast::StmtInfo<'a>>;

impl<'a> Parser<'a> {
    pub fn statement(&'a self) -> impl Fn(&'a str) -> IResultStmt<'a> {
        move |i: &str| self.expression_statement()(i)
    }

    fn expression_statement(&'a self) -> impl Fn(&'a str) -> IResultStmt<'a> {
        move |i: &str| {
            let (rest, expr) = self.expression()(i)?;
            let (rest, _) = primitive::symbol(";")(rest)?;

            Ok((
                rest,
                ast::StmtInfo {
                    kind: ast::StmtKind::Expr { expr },
                },
            ))
        }
    }
}

#[cfg(test)]
mod statement_parser_test {
    use super::*;

    #[test]
    fn statement_parser_test_main() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);

        let _ = expression_statement_test(&parser, "u100;", "");
        let _ = statement_test(&parser, "u100;", "");
    }

    fn statement_test<'a>(
        parser: &'a Parser<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::StmtInfo<'a> {
        let result = parser.statement()(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(rest, r);

        n
    }
    fn expression_statement_test<'a>(
        parser: &'a Parser<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::StmtInfo<'a> {
        let result = parser.expression_statement()(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(rest, r);

        n
    }
}
