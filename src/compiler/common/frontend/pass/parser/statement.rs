use crate::compiler::common::frontend::{
    pass::parser::{expression, parser::Parser, primitive},
    types::{allocator, ast},
};

use nom::character::complete::char as parse_char;

type IResultStmt<'a> = nom::IResult<&'a str, ast::StmtInfo<'a>>;

impl<'a> Parser<'a> {
    pub fn statement(&'a self, i: &'a str) -> IResultStmt<'a> {
        self.expression_statement()(i)
    }

    fn expression_statement(&'a self) -> impl Fn(&'a str) -> IResultStmt<'a> {
        move |i: &str| {
            let (rest, expr) = self.expression(i)?;
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
mod expression_parser_test {
    use super::*;

    #[test]
    fn expression_statement_test() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);
        let result = parser.expression_statement()("\"String\";");
        assert!(result.is_ok());

        let (rest, n) = result.unwrap();
        assert_eq!(
            ast::StmtInfo {
                kind: ast::StmtKind::Expr {
                    expr: ast::ExprInfo {
                        kind: ast::ExprKind::StringLiteral {
                            contents: "String".to_string(),
                        }
                    },
                },
            },
            n,
        );

        assert_eq!("", rest);
    }
}
