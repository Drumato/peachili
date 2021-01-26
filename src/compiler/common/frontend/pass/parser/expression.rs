use std::{cell::RefCell, rc::Rc};

use super::primitive;
use crate::compiler::common::frontend::types::ast;
use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::char as parse_char,
    combinator::{map, value},
    multi::many0,
    sequence::{preceded, tuple},
    IResult,
};

type IResultExpr<'a> = IResult<&'a str, ast::Expr>;

pub fn expression<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| alt((assignment(), addition()))(i)
}

fn assignment<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, var_name) = primitive::identifier_string()(i)?;
        let (rest, _) = primitive::symbol("=")(rest)?;
        let (rest, expr) = addition()(rest)?;

        Ok((
            rest,
            ast::Expr {
                kind: ast::ExprKind::Assignment {
                    var_name,
                    expr: ast::Expr::new_edge(expr),
                },
            },
        ))
    }
}

/// multiplication ('+' multiplication | '-' multiplication)*
fn addition<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, (mut head, tails)) = tuple((
            multiplication(),
            many0(tuple((
                alt((primitive::symbol("+"), primitive::symbol("-"))),
                multiplication(),
            ))),
        ))(i)?;

        for (op, tail_expr) in tails {
            match op {
                "+" => {
                    head = ast::Expr {
                        kind: ast::ExprKind::Addition {
                            lhs: ast::Expr::new_edge(head),
                            rhs: ast::Expr::new_edge(tail_expr),
                        },
                    }
                }
                "-" => {
                    head = ast::Expr {
                        kind: ast::ExprKind::Subtraction {
                            lhs: ast::Expr::new_edge(head),
                            rhs: ast::Expr::new_edge(tail_expr),
                        },
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok((rest, head))
    }
}

/// prefix ('*' prefix | '/' prefix)*
fn multiplication<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, (mut head, tails)) = tuple((
            prefix(),
            many0(tuple((
                alt((primitive::symbol("*"), primitive::symbol("/"))),
                prefix(),
            ))),
        ))(i)?;

        for (op, tail_expr) in tails {
            match op {
                "*" => {
                    head = ast::Expr {
                        kind: ast::ExprKind::Multiplication {
                            lhs: ast::Expr::new_edge(head),
                            rhs: ast::Expr::new_edge(tail_expr),
                        },
                    }
                }
                "/" => {
                    head = ast::Expr {
                        kind: ast::ExprKind::Division {
                            lhs: ast::Expr::new_edge(head),
                            rhs: ast::Expr::new_edge(tail_expr),
                        },
                    }
                }
                _ => unreachable!(),
            }
        }

        Ok((rest, head))
    }
}

/// unary_plus | unary_minus | primary
fn prefix<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| alt((unary_plus(), unary_minus(), primary()))(i)
}
/// "+" primary
fn unary_plus<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| preceded(primitive::symbol("+"), primary())(i)
}
/// "-" primary
fn unary_minus<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, child_node) = preceded(primitive::symbol("-"), primary())(i)?;
        Ok((
            rest,
            ast::Expr {
                kind: ast::ExprKind::Negative {
                    child: gen_child_node(child_node),
                },
            },
        ))
    }
}

/// string_literal | integer_literal | unsigned_integer_literal | identifier_sequence | boolean_literal
fn primary<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        alt((
            string_literal(),
            unsigned_integer_literal(),
            integer_literal(),
            boolean_literal(),
            identifier_expr(),
        ))(i)
    }
}

/// " character* "
fn string_literal<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, contents) = primitive::string_literal_string()(i)?;

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
/// identifier ("::" identifier)*
fn identifier_sequence<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, ident_list) = primitive::identifier_list_string()(i)?;
        Ok((
            rest,
            ast::Expr {
                kind: ast::ExprKind::Identifier { list: ident_list },
            },
        ))
    }
}

/// identifier_sequence parameter_list?
fn identifier_expr<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        let (rest, ident) = identifier_sequence()(i)?;

        match parameter_list()(rest) {
            Ok((rest, params)) => gen_result_primary(
                rest,
                ast::ExprKind::Call {
                    ident: gen_child_node(ident),
                    params,
                },
            ),
            Err(_e) => Ok((rest, ident)),
        }
    }
}

/// '(' list[expression, ','] ')'
fn parameter_list<'a>() -> impl Fn(&'a str) -> IResult<&str, Vec<ast::Expr>> {
    move |i: &str| {
        primitive::list_structure(primitive::Delimiter::Paren, ",", |i2: &str| {
            let (rest, n) = expression()(i2)?;
            Ok((rest, n))
        })(i)
    }
}

/// "[0-9]+"
fn integer_literal<'a>() -> impl Fn(&'a str) -> IResultExpr<'a> {
    move |i: &str| {
        map(primitive::ws(integer_literal_string()), |s: &str| {
            ast::Expr {
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

fn gen_result_primary<'a>(rest: &'a str, k: ast::ExprKind) -> IResultExpr<'a> {
    Ok((rest, ast::Expr { kind: k }))
}

fn gen_child_node(child: ast::Expr) -> Rc<RefCell<ast::Expr>> {
    Rc::new(RefCell::new(child))
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn assignment_test() {
        helper(
            assignment(),
            "x = 30;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Assignment {
                    var_name: "x".to_string(),
                    expr: ast::Expr::new_edge(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 30 },
                    }),
                },
            },
        );
    }

    #[test]
    fn addition_test() {
        helper(
            addition(),
            "1 + 1;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Addition {
                    lhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 1 },
                    })),
                    rhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 1 },
                    })),
                },
            },
        );
        helper(
            addition(),
            "1 - 1;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Subtraction {
                    lhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 1 },
                    })),
                    rhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 1 },
                    })),
                },
            },
        );
        helper(
            addition(),
            "1 + 2 - 3;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Subtraction {
                    lhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Addition {
                            lhs: Rc::new(RefCell::new(ast::Expr {
                                kind: ast::ExprKind::Integer { value: 1 },
                            })),
                            rhs: Rc::new(RefCell::new(ast::Expr {
                                kind: ast::ExprKind::Integer { value: 2 },
                            })),
                        },
                    })),
                    rhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 3 },
                    })),
                },
            },
        );
    }
    #[test]
    fn multiplication_test() {
        helper(
            multiplication(),
            "1 * 2;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Multiplication {
                    lhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 1 },
                    })),
                    rhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 2 },
                    })),
                },
            },
        );

        helper(
            multiplication(),
            "100 / 2;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Division {
                    lhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 100 },
                    })),
                    rhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 2 },
                    })),
                },
            },
        );
        helper(
            multiplication(),
            "100 / 2 / 10;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Division {
                    lhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Division {
                            lhs: Rc::new(RefCell::new(ast::Expr {
                                kind: ast::ExprKind::Integer { value: 100 },
                            })),
                            rhs: Rc::new(RefCell::new(ast::Expr {
                                kind: ast::ExprKind::Integer { value: 2 },
                            })),
                        },
                    })),
                    rhs: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 10 },
                    })),
                },
            },
        );
    }

    #[test]
    fn unary_plus_test() {
        helper(
            unary_plus(),
            "+ 100;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Integer { value: 100 },
            },
        );
    }
    #[test]
    fn unary_minus_test() {
        helper(
            unary_minus(),
            "- 100;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Negative {
                    child: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Integer { value: 100 },
                    })),
                },
            },
        );
    }

    #[test]
    fn string_literal_test() {
        helper(
            string_literal(),
            "\"Hello, world!\";",
            ";",
            ast::Expr {
                kind: ast::ExprKind::StringLiteral {
                    contents: "Hello, world!".to_string(),
                },
            },
        );
    }

    #[test]
    fn primary_test() {
        helper(
            primary(),
            "true;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::True,
            },
        );
        helper(
            primary(),
            "100;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Integer { value: 100 },
            },
        );
        helper(
            primary(),
            "u100;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::UnsignedInteger { value: 100 },
            },
        );
    }

    #[test]
    fn boolean_literal_test() {
        helper(
            boolean_literal(),
            "true;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::True,
            },
        );
        helper(
            boolean_literal(),
            "false;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::False,
            },
        );
    }

    #[test]
    fn identifier_expr_test() {
        helper(
            identifier_expr(),
            "x64::exit_with(0, 1, 2, 3);",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Call {
                    ident: Rc::new(RefCell::new(ast::Expr {
                        kind: ast::ExprKind::Identifier {
                            list: vec!["x64".to_string(), "exit_with".to_string()],
                        },
                    })),
                    params: vec![
                        ast::Expr {
                            kind: ast::ExprKind::Integer { value: 0 },
                        },
                        ast::Expr {
                            kind: ast::ExprKind::Integer { value: 1 },
                        },
                        ast::Expr {
                            kind: ast::ExprKind::Integer { value: 2 },
                        },
                        ast::Expr {
                            kind: ast::ExprKind::Integer { value: 3 },
                        },
                    ],
                },
            },
        );
    }

    #[test]
    fn identifier_sequence_test() {
        helper(
            identifier_sequence(),
            "drumato;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Identifier {
                    list: vec!["drumato".to_string()],
                },
            },
        );
        helper(
            identifier_sequence(),
            "x86_64::STDIN;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Identifier {
                    list: vec!["x86_64".to_string(), "STDIN".to_string()],
                },
            },
        );
    }

    #[test]
    fn unsigned_integer_literal_test() {
        helper(
            unsigned_integer_literal(),
            "u100;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::UnsignedInteger { value: 100 },
            },
        );
    }

    #[test]
    fn integer_literal_test() {
        helper(
            integer_literal(),
            "100;",
            ";",
            ast::Expr {
                kind: ast::ExprKind::Integer { value: 100 },
            },
        );
    }

    fn helper<'a>(
        f: impl Fn(&'a str) -> IResultExpr<'a>,
        input: &'a str,
        rest: &'a str,
        expected: ast::Expr,
    ) {
        let result = f(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();
        assert_eq!(rest, r);

        assert_eq!(expected, n);
    }
}
