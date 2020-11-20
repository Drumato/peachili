use std::cell::RefCell;

use super::parser::Parser;
use super::primitive;
use crate::compiler::common::frontend::types::{allocator, ast};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char as parse_char, multispace0},
    combinator::{map, value},
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
};

type IResultExpr<'a> = nom::IResult<&'a str, ast::ExprInfo<'a>>;

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

    /// [a-zA-Z] ('_' | [a-zA-Z0-9])*
    fn identifier_string(&'a self) -> impl Fn(&'a str) -> nom::IResult<&'a str, String> {
        move |i: &str| {
            let (rest, (_, head, last, _)) = tuple((
                multispace0,
                take_while1(|b: char| b.is_alphabetic()),
                take_while(|b: char| b.is_alphanumeric() || b == '_'),
                multispace0,
            ))(i)?;
            Ok((rest, format!("{}{}", head, last)))
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

    fn integer_literal_string(&'a self) -> impl Fn(&'a str) -> nom::IResult<&str, &str> {
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
    fn minus_operation_test() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);
        let result = parser.minus_operation("- 100;");
        assert!(result.is_ok());

        let (rest, n) = result.unwrap();
        assert_eq!(
            ast::ExprInfo {
                kind: ast::ExprKind::Negative {
                    child: RefCell::new(&ast::ExprInfo {
                        kind: ast::ExprKind::Integer { value: 100 },
                    }),
                },
            },
            n,
        );

        assert_eq!(";", rest);
    }

    #[test]
    fn primary_test() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);
        let result = parser.primary()(" u100 Drumato;");
        assert!(result.is_ok());

        let (rest, literal) = result.unwrap();
        assert_eq!(
            ast::ExprInfo {
                kind: ast::ExprKind::UnsignedInteger { value: 100 },
            },
            literal
        );
        assert_eq!("Drumato;", rest);
        let result = parser.primary()(rest);
        assert!(result.is_ok());

        let (rest, literal) = result.unwrap();
        assert_eq!(
            ast::ExprInfo {
                kind: ast::ExprKind::Identifier {
                    list: vec!["Drumato".to_string()],
                },
            },
            literal
        );

        assert_eq!(";", rest);
    }

    #[test]
    fn string_literal_test() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);
        let result = parser.string_literal()("\"Hello, world!\";");
        assert!(result.is_ok());

        let (rest, literal) = result.unwrap();
        assert_eq!(
            ast::ExprInfo {
                kind: ast::ExprKind::StringLiteral {
                    contents: "Hello, world!".to_string()
                },
            },
            literal
        );
        assert_eq!(";", rest);
    }

    #[test]
    fn boolean_literal_test() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);
        let result = parser.boolean_literal()(" true false ;");
        assert!(result.is_ok());

        let (rest, literal) = result.unwrap();
        assert_eq!(
            ast::ExprInfo {
                kind: ast::ExprKind::True
            },
            literal
        );
        assert_eq!("false ;", rest);

        let result = parser.boolean_literal()(rest);
        assert!(result.is_ok());

        let (rest, literal) = result.unwrap();
        assert_eq!(
            ast::ExprInfo {
                kind: ast::ExprKind::False
            },
            literal
        );
        assert_eq!(";", rest);
    }

    #[test]
    fn identifier_sequence_test() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);
        let result = parser.identifier_sequence()("    drumato;");
        assert_eq!(
            Ok((
                ";",
                ast::ExprInfo {
                    kind: ast::ExprKind::Identifier {
                        list: vec!["drumato".to_string()],
                    }
                },
            )),
            result
        );

        let result = parser.identifier_sequence()("   x64::STDIN;");
        assert_eq!(
            Ok((
                ";",
                ast::ExprInfo {
                    kind: ast::ExprKind::Identifier {
                        list: vec!["x64".to_string(), "STDIN".to_string()],
                    }
                },
            )),
            result
        );
    }

    #[test]
    fn identifier_string_with_invalid_input() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);
        let result = parser.identifier_string()("100drumato;");
        assert!(result.is_err());
        let result = parser.identifier_string()("drumato;");
        assert_eq!(Ok((";", "drumato".to_string())), result);
        let result = parser.identifier_string()("100yen;");
        assert!(result.is_err());
        let result = parser.identifier_string()("foo1;");
        assert_eq!(Ok((";", "foo1".to_string())), result);
        let result = parser.identifier_string()("foo_1;");
        assert_eq!(Ok((";", "foo_1".to_string())), result);
    }

    #[test]
    fn unsigned_integer_literal_test() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);
        let result = parser.unsigned_integer_literal()("   u300;");
        assert_eq!(
            Ok((
                ";",
                ast::ExprInfo {
                    kind: ast::ExprKind::UnsignedInteger { value: 300 }
                }
            )),
            result
        );
    }

    #[test]
    fn integer_literal_test() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);
        let result = parser.integer_literal()("   300;");
        assert_eq!(
            Ok((
                ";",
                ast::ExprInfo {
                    kind: ast::ExprKind::Integer { value: 300 }
                }
            )),
            result
        );
    }

    #[test]
    fn integer_literal_string_with_invalid_input() {
        let arena = Default::default();
        let parser: Parser = Parser::new(&arena);
        let f = parser.integer_literal_string();
        let result = f("   abc;");
        assert!(result.is_err());
        let result = f("   u100;");
        assert!(result.is_err());
    }
}
