use std::cell::RefCell;

use super::primitive;
use super::Parser;
use crate::compiler::common::frontend::types::ast;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::char as parse_char,
    combinator::{map, value},
    multi::separated_list0,
    sequence::{delimited, preceded},
    IResult,
};

type IResultExpr<'a> = IResult<&'a str, ast::ExprInfo<'a>>;

impl<'a> Parser<'a> {
    pub fn expression(&'a self, i: &'a str) -> IResultExpr<'a> {
        self.postfix(i)
    }

    /// minus_operation | primary
    fn postfix(&'a self, i: &'a str) -> IResultExpr<'a> {
        match i.as_bytes()[0] {
            // '-' 's ascii
            45 => self.minus_operation(i),
            _ => self.primary()(i),
        }
    }
    /// "-" primary
    fn minus_operation(&'a self, i: &'a str) -> IResultExpr<'a> {
        let (rest, child_node) = preceded(parse_char('-'), self.primary())(i)?;
        Ok((
            rest,
            ast::ExprInfo {
                kind: ast::ExprKind::Negative {
                    child: self.gen_child_node(child_node),
                },
            },
        ))
    }

    /// string_literal | integer_literal | unsigned_integer_literal | identifier_sequence | boolean_literal
    fn primary(&'a self) -> impl Fn(&'a str) -> IResultExpr<'a> {
        move |i: &str| {
            alt((
                self.string_literal(),
                self.unsigned_integer_literal(),
                self.integer_literal(),
                self.boolean_literal(),
                self.identifier_sequence(),
            ))(i)
        }
    }

    /// " character* "
    fn string_literal(&'a self) -> impl Fn(&'a str) -> IResultExpr<'a> {
        move |i: &str| {
            let (rest, contents) = primitive::ws(delimited(
                primitive::symbol("\""),
                take_while(|b: char| b != '"'),
                primitive::symbol("\""),
            ))(i)?;
            self.gen_result_primary(
                rest,
                ast::ExprKind::StringLiteral {
                    contents: contents.to_string(),
                },
            )
        }
    }

    /// "true" | "false"
    fn boolean_literal(&'a self) -> impl Fn(&'a str) -> IResultExpr<'a> {
        move |i: &str| {
            let (rest, literal_kind) = alt((
                value(ast::ExprKind::True, primitive::keyword("true")),
                value(ast::ExprKind::False, primitive::keyword("false")),
            ))(i)?;
            self.gen_result_primary(rest, literal_kind)
        }
    }

    /// identnfier ("::" identifier)*
    fn identifier_sequence(&'a self) -> impl Fn(&'a str) -> IResultExpr<'a> {
        move |i: &str| {
            let (rest, ident_list) = separated_list0(tag("::"), self.identifier_string())(i)?;
            self.gen_result_primary(rest, ast::ExprKind::Identifier { list: ident_list })
        }
    }

    /// "[0-9]+"
    fn integer_literal(&'a self) -> impl Fn(&'a str) -> IResultExpr<'a> {
        move |i: &str| {
            map(primitive::ws(self.integer_literal_string()), |s: &str| {
                ast::ExprInfo {
                    kind: ast::ExprKind::Integer {
                        value: s.parse().unwrap(),
                    },
                }
            })(i)
        }
    }

    /// "'u' [0-9]+"
    fn unsigned_integer_literal(&'a self) -> impl Fn(&'a str) -> IResultExpr<'a> {
        move |i: &str| {
            let (rest, unsigned_integer_string) =
                primitive::ws(preceded(parse_char('u'), self.integer_literal_string()))(i)?;

            self.gen_result_primary(
                rest,
                ast::ExprKind::UnsignedInteger {
                    value: unsigned_integer_string.parse().unwrap(),
                },
            )
        }
    }

    fn integer_literal_string(&'a self) -> impl Fn(&'a str) -> IResult<&str, &str> {
        move |i: &str| take_while1(|b: char| b.is_ascii_digit())(i)
    }

    fn gen_result_primary(&'a self, rest: &'a str, k: ast::ExprKind<'a>) -> IResultExpr<'a> {
        Ok((rest, ast::ExprInfo { kind: k }))
    }

    fn gen_child_node(&'a self, child: ast::ExprInfo<'a>) -> RefCell<ast::Expr<'a>> {
        RefCell::new(self.allocator.expr_arena.alloc(child))
    }
}

#[cfg(test)]
mod expression_parser_test {
    use super::*;

    #[test]
    fn expression_parser_test_main() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);

        let _ = minus_operation_test(&parser, "- 100;", ";");
        let _ = primary_test(&parser, "u100;", ";");
        let _ = string_literal_test(&parser, "\"Hello, world!\";", ";");
        let _ = boolean_literal_test(&parser, "true;", ";");
        let _ = identifier_sequence_test(&parser, "    drumato;", ";");
        let _ = identifier_sequence_test(&parser, "    x64::STDIN;", ";");
        let _ = identifier_string_with_invalid_input(&parser, "100yen;");
        let _ = identifier_string_with_invalid_input(&parser, "!foafekajl;");
        let _ = unsigned_integer_literal_test(&parser, "    u300;", ";");
        let _ = integer_literal_test(&parser, "100;", ";");
    }

    fn minus_operation_test<'a>(
        parser: &'a Parser<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::ExprInfo<'a> {
        let result = parser.minus_operation(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(rest, r);

        n
    }

    fn primary_test<'a>(
        parser: &'a Parser<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::ExprInfo<'a> {
        let result = parser.primary()(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(rest, r);

        n
    }
    fn string_literal_test<'a>(
        parser: &'a Parser<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::ExprInfo<'a> {
        let result = parser.string_literal()(input);
        assert!(result.is_ok());

        let (r, literal) = result.unwrap();
        assert_eq!(rest, r);

        literal
    }

    fn boolean_literal_test<'a>(
        parser: &'a Parser<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::ExprInfo<'a> {
        let result = parser.boolean_literal()(input);
        assert!(result.is_ok());

        let (r, literal) = result.unwrap();
        assert_eq!(rest, r);

        literal
    }

    fn identifier_sequence_test<'a>(
        parser: &'a Parser<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::ExprInfo<'a> {
        let result = parser.identifier_sequence()(input);
        assert!(result.is_ok());

        let (r, literal) = result.unwrap();
        assert_eq!(rest, r);

        literal
    }

    fn identifier_string_with_invalid_input<'a>(parser: &'a Parser<'a>, input: &'a str) {
        let result = parser.identifier_string()(input);
        assert!(result.is_err());
    }

    fn unsigned_integer_literal_test<'a>(
        parser: &'a Parser<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::ExprInfo<'a> {
        let result = parser.unsigned_integer_literal()(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();
        assert_eq!(rest, r);

        n
    }

    fn integer_literal_test<'a>(
        parser: &'a Parser<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::ExprInfo<'a> {
        let result = parser.integer_literal()(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();
        assert_eq!(rest, r);

        n
    }
}
