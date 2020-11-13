use std::cell::RefCell;
use typed_arena::Arena;
use crate::common::analyze_resource::ast;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::char as parse_char,
    multi::{separated_list0},
    sequence::{delimited, pair, preceded},
    combinator::{map, value},
};

type IResultExpr<'a> = nom::IResult<&'a str, ast::ExprInfo<'a>>;

/// "-" primary
fn minus_operation<'a>(arena: &'a Arena<ast::ExprInfo<'a>>, i: &'a str) -> IResultExpr<'a> {
    let (rest, child_node) = preceded(parse_char('-'), primary)(i)?;
    Ok((
        rest,
        ast::ExprInfo{kind: ast::ExprKind::Negative{child: gen_child_node(arena, child_node)}},
    ))
}

/// string_literal | integer_literal | unsigned_integer_literal | identifier_sequence | boolean_literal
fn primary(i: &str) -> IResultExpr {
    alt((string_literal, unsigned_integer_literal, integer_literal, boolean_literal, identifier_sequence))(i)
}

/// " character* "
fn string_literal(i: &str) -> IResultExpr {
    let (rest, contents) = delimited(
        parse_char('"'),
        take_while(|b: char| b != '"'),
        parse_char('"'),
    )(i)?;

    gen_result_primary(
        rest,
        ast::ExprKind::StringLiteral {
            contents: contents.to_string(),
        },
    )
}

/// "true" | "false"
fn boolean_literal(i: &str) -> IResultExpr {
    let (rest, literal_kind) = alt(
        (
            value(ast::ExprKind::True, tag("true")), 
            value(ast::ExprKind::False, tag("false")),
        )
    )(i)?;
    gen_result_primary(rest, literal_kind)
}

/// identnfier ("::" identifier)*
fn identifier_sequence(i: &str) -> IResultExpr {
    let (rest, ident_list) = separated_list0(tag("::"), identifier_string)(i)?;
    gen_result_primary(rest, ast::ExprKind::Identifier { list: ident_list })
}

/// [a-zA-Z] ('_' | [a-zA-Z0-9])*
fn identifier_string(i: &str) -> nom::IResult<&str, String> {
    let (rest, (head, last)) = pair(
        take_while1(|b: char| b.is_alphabetic()),
        take_while(|b: char| b.is_alphanumeric() || b == '_'),
    )(i)?;

    Ok((rest, format!("{}{}", head, last)))
}

/// "[0-9]+"
fn integer_literal(i: &str) -> IResultExpr {
    map(integer_literal_string, |s: &str| ast::ExprInfo{ kind: ast::ExprKind::Integer{value: s.parse().unwrap()}})(i)
}

/// "'u' [0-9]+"
fn unsigned_integer_literal(i: &str) -> IResultExpr {
    let (rest, unsigned_integer_string) = preceded(parse_char('u'), integer_literal_string)(i)?;

    gen_result_primary(
        rest,
        ast::ExprKind::UnsignedInteger {
            value: unsigned_integer_string.parse().unwrap(),
        },
    )
}

fn integer_literal_string(i: &str) -> nom::IResult<&str, &str> {
    take_while1(|b: char| b.is_ascii_digit())(i)
}

fn gen_result_primary<'a>(rest: &'a str, k: ast::ExprKind<'a>) -> IResultExpr<'a> {
    Ok((rest, ast::ExprInfo { kind: k }))
}

fn gen_child_node<'a>(arena: &'a Arena<ast::ExprInfo<'a>>, child: ast::ExprInfo<'a>) -> RefCell<ast::Expr<'a>> {
    RefCell::new(arena.alloc(child))
}

#[cfg(test)]
mod expression_parser_test {
    use super::*;

    #[test]
    fn minus_operation_test() {
        let arena = Arena::new();
        let result = minus_operation(&arena, "-100;");
        assert!(result.is_ok());

        let (rest, n) = result.unwrap();
        assert_eq!(
            ast::ExprInfo {
                kind: ast::ExprKind::Negative{
                    child: RefCell::new(&ast::ExprInfo{
                        kind: ast::ExprKind::Integer{value: 100},
                    }),
                },
            },
            n,
        );

        assert_eq!(";", rest);
    }

    #[test]
    fn primary_test() {
        let result = primary("u100Drumato;");
        assert!(result.is_ok());

        let (rest, literal) = result.unwrap();
        assert_eq!(
            ast::ExprInfo {
                kind: ast::ExprKind::UnsignedInteger {
                    value: 100,
                },
            },
            literal
        );
        assert_eq!("Drumato;", rest);
        let result = primary(rest);
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
        let result = string_literal("\"Hello, world!\";");
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
        let result = boolean_literal("truefalse;");
        assert!(result.is_ok());

        let (rest, literal) = result.unwrap();
        assert_eq!(
            ast::ExprInfo {
                kind: ast::ExprKind::True
            },
            literal
        );
        assert_eq!("false;", rest);

        let result = boolean_literal(rest);
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
        let result = identifier_sequence("drumato;");
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

        let result = identifier_sequence("x64::STDIN;");
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
        let result = identifier_string("100drumato;");
        assert!(result.is_err());
        let result = identifier_string("drumato;");
        assert_eq!(Ok((";", "drumato".to_string())), result);
        let result = identifier_string("100yen;");
        assert!(result.is_err());
        let result = identifier_string("foo1;");
        assert_eq!(Ok((";", "foo1".to_string())), result);
        let result = identifier_string("foo_1;");
        assert_eq!(Ok((";", "foo_1".to_string())), result);
    }

    #[test]
    fn unsigned_integer_literal_test() {
        let result = unsigned_integer_literal("u300;");
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
        let result = integer_literal("300;");
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
        let result = integer_literal_string("abc;");
        assert!(result.is_err());
        let result = integer_literal_string("u100;");
        assert!(result.is_err());
    }
}
