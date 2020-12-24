use std::cell::RefCell;

use super::primitive;
use crate::compiler::common::frontend::{allocator::Allocator, types::ast};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char as parse_char,
    combinator::{map, value},
    multi::separated_list0,
    sequence::preceded,
    IResult,
};

type IResultExpr<'a> = IResult<&'a str, ast::ExprInfo<'a>>;

pub fn expression<'a>(alloc: &'a Allocator<'a>) -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| postfix(alloc, i)
}

/// minus_operation | primary
fn postfix<'a>(alloc: &'a Allocator<'a>, i: &'a str) -> IResultExpr<'a> {
    match i.as_bytes()[0] {
        // '-' 's ascii
        45 => minus_operation(alloc, i),
        _ => primary(alloc)(i),
    }
}
/// "-" primary
fn minus_operation<'a>(alloc: &'a Allocator<'a>, i: &'a str) -> IResultExpr<'a> {
    let (rest, child_node) = preceded(parse_char('-'), primary(alloc))(i)?;
    Ok((
        rest,
        ast::ExprInfo {
            kind: ast::ExprKind::Negative {
                child: gen_child_node(alloc, child_node),
            },
        },
    ))
}

/// string_literal | integer_literal | unsigned_integer_literal | identifier_sequence | boolean_literal
fn primary<'a>(alloc: &'a Allocator<'a>) -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        alt((
            string_literal(),
            unsigned_integer_literal(),
            integer_literal(),
            boolean_literal(),
            identifier_expr(alloc),
        ))(i)
    }
}

/// " character* "
fn string_literal<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, contents) = primitive::string_literal_str()(i)?;

        gen_result_primary(
            rest,
            ast::ExprKind::StringLiteral {
                contents: contents.to_string(),
            },
        )
    }
}

/// "true" | "false"
fn boolean_literal<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, literal_kind) = alt((
            value(ast::ExprKind::True, primitive::keyword("true")),
            value(ast::ExprKind::False, primitive::keyword("false")),
        ))(i)?;
        gen_result_primary(rest, literal_kind)
    }
}

/// identifier_sequence args_list?
fn identifier_expr<'a>(alloc: &'a Allocator<'a>) -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, ident) = identifier_sequence()(i)?;

        if rest.as_bytes()[0] != '(' as u8 {
            return Ok((rest, ident));
        }

        let (rest, args) = argument_list(alloc)(rest)?;

        gen_result_primary(
            rest,
            ast::ExprKind::Call {
                ident: gen_child_node(alloc, ident),
                args,
            },
        )
    }
}

/// '(' list[expression, ','] ')'
fn argument_list<'a>(
    alloc: &'a Allocator<'a>,
) -> impl Fn(&'a str) -> IResult<&str, Vec<RefCell<ast::Expr<'a>>>> {
    move |i: &str| {
        primitive::list_structure(primitive::Delimiter::Paren, ",", |i2: &str| {
            let (rest, n) = expression(alloc)(i2)?;
            Ok((rest, gen_child_node(alloc, n)))
        })(i)
    }
}

/// identnfier ("::" identifier)*
fn identifier_sequence<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, ident_list) = separated_list0(tag("::"), primitive::identifier_string())(i)?;
        gen_result_primary(rest, ast::ExprKind::Identifier { list: ident_list })
    }
}

/// "[0-9]+"
fn integer_literal<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        map(primitive::ws(integer_literal_string()), |s: &str| {
            ast::ExprInfo {
                kind: ast::ExprKind::Integer {
                    value: s.parse().unwrap(),
                },
            }
        })(i)
    }
}

/// "'u' [0-9]+"
fn unsigned_integer_literal<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, unsigned_integer_string) =
            primitive::ws(preceded(parse_char('u'), integer_literal_string()))(i)?;

        gen_result_primary(
            rest,
            ast::ExprKind::UnsignedInteger {
                value: unsigned_integer_string.parse().unwrap(),
            },
        )
    }
}

fn integer_literal_string<'a>() -> impl Fn(&'a str) -> IResult<&str, &str> {
    move |i: &str| take_while1(|b: char| b.is_ascii_digit())(i)
}

fn gen_result_primary<'a>(rest: &'a str, k: ast::ExprKind<'a>) -> IResultExpr<'a> {
    Ok((rest, ast::ExprInfo { kind: k }))
}

fn gen_child_node<'a>(
    alloc: &'a Allocator<'a>,
    child: ast::ExprInfo<'a>,
) -> RefCell<ast::Expr<'a>> {
    RefCell::new(alloc.expr_arena.alloc(child))
}
#[cfg(test)]
mod expression_parser_test {
    use super::*;

    #[test]
    fn expression_parser_test_main() {
        let arena = Default::default();

        let _ = minus_operation_test(&arena, "- 100;", ";");
        let _ = primary_test(&arena, "u100;", ";");
        let _ = string_literal_test("\"Hello, world!\";", ";");
        let _ = boolean_literal_test("true;", ";");
        let _ = identifier_sequence_test("    drumato;", ";");
        let _ = identifier_sequence_test("    x64::STDIN;", ";");
        let _ = unsigned_integer_literal_test("    u300;", ";");
        let _ = integer_literal_test("100;", ";");
        let call_expr = identifier_expr_test(&arena, "x64::exit_with(0, 1, 2, 3);", ";");
        assert_eq!(
            ast::ExprInfo {
                kind: ast::ExprKind::Call {
                    ident: RefCell::new(&ast::ExprInfo {
                        kind: ast::ExprKind::Identifier {
                            list: vec!["x64".to_string(), "exit_with".to_string()]
                        }
                    }),
                    args: vec![
                        RefCell::new(&ast::ExprInfo {
                            kind: ast::ExprKind::Integer { value: 0 }
                        }),
                        RefCell::new(&ast::ExprInfo {
                            kind: ast::ExprKind::Integer { value: 1 }
                        }),
                        RefCell::new(&ast::ExprInfo {
                            kind: ast::ExprKind::Integer { value: 2 }
                        }),
                        RefCell::new(&ast::ExprInfo {
                            kind: ast::ExprKind::Integer { value: 3 }
                        }),
                    ],
                }
            },
            call_expr
        );
    }

    fn minus_operation_test<'a>(
        alloc: &'a Allocator<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::ExprInfo<'a> {
        let result = minus_operation(alloc, input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(rest, r);

        n
    }

    fn primary_test<'a>(
        alloc: &'a Allocator<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::ExprInfo<'a> {
        let result = primary(alloc)(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(rest, r);

        n
    }
    fn string_literal_test<'a>(input: &'a str, rest: &'a str) -> ast::ExprInfo<'a> {
        let result = string_literal()(input);
        assert!(result.is_ok());

        let (r, literal) = result.unwrap();
        assert_eq!(rest, r);

        literal
    }

    fn boolean_literal_test<'a>(input: &'a str, rest: &'a str) -> ast::ExprInfo<'a> {
        let result = boolean_literal()(input);
        assert!(result.is_ok());

        let (r, literal) = result.unwrap();
        assert_eq!(rest, r);

        literal
    }

    fn identifier_expr_test<'a>(
        alloc: &'a Allocator<'a>,
        input: &'a str,
        rest: &'a str,
    ) -> ast::ExprInfo<'a> {
        let result = identifier_expr(alloc)(input);
        assert!(result.is_ok());

        let (r, literal) = result.unwrap();
        assert_eq!(rest, r);

        literal
    }

    fn identifier_sequence_test<'a>(input: &'a str, rest: &'a str) -> ast::ExprInfo<'a> {
        let result = identifier_sequence()(input);
        assert!(result.is_ok());

        let (r, literal) = result.unwrap();
        assert_eq!(rest, r);

        literal
    }

    fn unsigned_integer_literal_test<'a>(input: &'a str, rest: &'a str) -> ast::ExprInfo<'a> {
        let result = unsigned_integer_literal()(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();
        assert_eq!(rest, r);

        n
    }

    fn integer_literal_test<'a>(input: &'a str, rest: &'a str) -> ast::ExprInfo<'a> {
        let result = integer_literal()(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();
        assert_eq!(rest, r);

        n
    }
}
