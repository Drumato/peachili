use crate::common::analyze_resource::ast;
use nom::{
    bytes::complete::{tag, take_while, take_while1},
    character::complete::char as parse_char,
    multi::separated_list0,
    sequence::{pair, preceded},
};

/// identnfier ("::" identifier)*
fn identifier_sequence(i: &str) -> nom::IResult<&str, ast::Expr> {
    let (rest, ident_list) = separated_list0(tag("::"), identifier_string)(i)?;
    Ok((
        rest,
        ast::Expr {
            kind: ast::ExprKind::Identifier { list: ident_list },
        },
    ))
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
fn integer_literal(i: &str) -> nom::IResult<&str, ast::Expr> {
    let (rest, integer_string) = integer_literal_string(i)?;

    Ok((
        rest,
        ast::Expr {
            kind: ast::ExprKind::Integer {
                value: integer_string.parse().unwrap(),
            },
        },
    ))
}

/// "'u' [0-9]+"
fn unsigned_integer_literal(i: &str) -> nom::IResult<&str, ast::Expr> {
    let (rest, unsigned_integer_string) = preceded(parse_char('u'), integer_literal_string)(i)?;

    Ok((
        rest,
        ast::Expr {
            kind: ast::ExprKind::UnsignedInteger {
                value: unsigned_integer_string.parse().unwrap(),
            },
        },
    ))
}

fn integer_literal_string(i: &str) -> nom::IResult<&str, &str> {
    take_while1(|b: char| b.is_ascii_digit())(i)
}

#[cfg(test)]
mod expression_parser_test {
    use super::*;

    #[test]
    fn identifier_sequence_test() {
        let result = identifier_sequence("drumato;");
        assert_eq!(
            Ok((
                ";",
                ast::Expr {
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
                ast::Expr {
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
                ast::Expr {
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
                ast::Expr {
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
