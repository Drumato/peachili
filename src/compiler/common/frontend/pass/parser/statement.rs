use crate::compiler::common::frontend::{pass::parser::*, types::ast};

use nom::{branch::alt, multi::many0, IResult};
use primitive::{keyword, list_structure, string_literal_string, symbol, Delimiter};

type IResultStmt<'a> = IResult<&'a str, ast::Stmt>;

pub fn statement<'a>() -> impl Fn(&'a str) -> IResultStmt<'a> {
    move |i: &str| {
        alt((
            countup_statement(),
            block_statement(),
            declaration_statement(),
            asm_statement(),
            expression_statement(),
        ))(i)
    }
}

/// "countup" identifier "from" expression ("lessthan" | "to") expression block_statement
fn countup_statement<'a>() -> impl Fn(&'a str) -> IResultStmt<'a> {
    move |i: &str| {
        let (rest, _) = primitive::keyword("countup")(i)?;
        let (rest, id) = primitive::identifier_string()(rest)?;
        let (rest, _) = primitive::keyword("from")(rest)?;
        let (rest, from_expr) = expression::expression()(rest)?;
        let (rest, to_key) = alt((primitive::keyword("lessthan"), primitive::keyword("to")))(rest)?;
        let (rest, end_expr) = expression::expression()(rest)?;
        let (rest, block) = compound_statement()(rest)?;

        let stmt = if to_key == "lessthan" {
            ast::Stmt {
                kind: ast::StmtKind::HalfOpenCountup {
                    block,
                    id,
                    from: from_expr,
                    lessthan: end_expr,
                },
            }
        } else {
            ast::Stmt {
                kind: ast::StmtKind::ClosedCountup {
                    block,
                    id,
                    from: from_expr,
                    to: end_expr,
                },
            }
        };

        Ok((rest, stmt))
    }
}

/// "{" statement* "}"
fn block_statement<'a>() -> impl Fn(&'a str) -> IResultStmt<'a> {
    move |i: &str| {
        let (rest, stmts) = compound_statement()(i)?;
        Ok((
            rest,
            ast::Stmt {
                kind: ast::StmtKind::Block { stmts },
            },
        ))
    }
}
/// "{" statement* "}"
fn compound_statement<'a>() -> impl Fn(&'a str) -> nom::IResult<&'a str, Vec<ast::Stmt>> {
    move |i: &str| {
        let (rest, _) = primitive::symbol("{")(i)?;
        let (rest, stmts) = many0(statement())(rest)?;
        let (rest, _) = primitive::symbol("}")(rest)?;

        Ok((rest, stmts))
    }
}

/// "declare" identifier type-name ';'
fn declaration_statement<'a>() -> impl Fn(&'a str) -> IResultStmt<'a> {
    move |i: &str| {
        let (rest, _) = primitive::keyword("declare")(i)?;
        let (rest, var_name) = primitive::identifier_string()(rest)?;
        let (rest, type_name) = primitive::identifier_list_string()(rest)?;
        let (rest, _) = primitive::symbol(";")(rest)?;

        Ok((
            rest,
            ast::Stmt {
                kind: ast::StmtKind::Declare {
                    var_name,
                    type_name,
                },
            },
        ))
    }
}

/// expression ';'
fn expression_statement<'a>() -> impl Fn(&'a str) -> IResultStmt<'a> {
    move |i: &str| {
        let (rest, expr) = expression()(i)?;
        let (rest, _) = primitive::symbol(";")(rest)?;

        Ok((
            rest,
            ast::Stmt {
                kind: ast::StmtKind::Expr { expr },
            },
        ))
    }
}

/// "asm" '{' list[string_literal, ';'] '}' ';'
fn asm_statement<'a>() -> impl Fn(&'a str) -> IResultStmt<'a> {
    move |i: &str| {
        let (rest, _) = keyword("asm")(i)?;
        let (rest, asm_insts) =
            list_structure(Delimiter::Bracket, ";", string_literal_string())(rest)?;
        let (rest, _) = symbol(";")(rest)?;

        Ok((
            rest,
            ast::Stmt {
                kind: ast::StmtKind::Asm {
                    insts: asm_insts.iter().map(|s| s.to_string()).collect(),
                },
            },
        ))
    }
}

#[cfg(test)]
mod statement_parser_test {
    use super::*;

    #[test]
    fn statement_test() {
        let result =
            statement()("u100; asm { \"movq $60, %rax\"; \"syscall\" }; declare x Int64;;");
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(
            "asm { \"movq $60, %rax\"; \"syscall\" }; declare x Int64;;",
            r
        );
        assert_eq!(
            ast::Stmt {
                kind: ast::StmtKind::Expr {
                    expr: ast::Expr {
                        kind: ast::ExprKind::UnsignedInteger { value: 100 }
                    }
                }
            },
            n
        );

        let result = statement()(r);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();
        assert_eq!("declare x Int64;;", r);
        assert_eq!(
            ast::Stmt {
                kind: ast::StmtKind::Asm {
                    insts: vec!["movq $60, %rax".to_string(), "syscall".to_string()],
                }
            },
            n
        );
        let result = statement()(r);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();
        assert_eq!(";", r);
        assert_eq!(
            ast::Stmt {
                kind: ast::StmtKind::Declare {
                    var_name: "x".to_string(),
                    type_name: vec!["Int64".to_string()],
                }
            },
            n
        );
    }

    #[test]
    fn asm_statement_test() {
        let result = asm_statement()("asm { \"movq $60, %rax\"; \"syscall\" };;");
        assert!(result.is_ok());

        let (r, n) = result.unwrap();
        assert_eq!(";", r);
        assert_eq!(
            ast::Stmt {
                kind: ast::StmtKind::Asm {
                    insts: vec!["movq $60, %rax".to_string(), "syscall".to_string()],
                }
            },
            n
        );
    }

    #[test]
    fn declaration_statement_test() {
        let result = declaration_statement()("declare x Int64;;");
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(";", r);
        assert_eq!(
            ast::Stmt {
                kind: ast::StmtKind::Declare {
                    var_name: "x".to_string(),
                    type_name: vec!["Int64".to_string()],
                }
            },
            n
        );
    }
    #[test]
    fn expression_statement_test() {
        let result = expression_statement()("u100;;");
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(";", r);
        assert_eq!(
            ast::Stmt {
                kind: ast::StmtKind::Expr {
                    expr: ast::Expr {
                        kind: ast::ExprKind::UnsignedInteger { value: 100 }
                    }
                }
            },
            n
        );
    }

    #[test]
    fn block_statement_test() {
        let result = block_statement()("{ declare x Int64; x = 30; };");
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(";", r);
        assert_eq!(
            ast::Stmt {
                kind: ast::StmtKind::Block {
                    stmts: vec![
                        ast::Stmt {
                            kind: ast::StmtKind::Declare {
                                var_name: "x".to_string(),
                                type_name: vec!["Int64".to_string()],
                            }
                        },
                        ast::Stmt {
                            kind: ast::StmtKind::Expr {
                                expr: ast::Expr {
                                    kind: ast::ExprKind::Assignment {
                                        var_name: "x".to_string(),
                                        expr: ast::Expr::new_edge(ast::Expr {
                                            kind: ast::ExprKind::Integer { value: 30 }
                                        }),
                                    }
                                }
                            }
                        }
                    ],
                }
            },
            n
        );
    }

    #[test]
    fn countup_statement_test() {
        let result = countup_statement()("countup x from 0 lessthan 10 { x = 30; };");
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(";", r);
        assert_eq!(
            ast::Stmt {
                kind: ast::StmtKind::HalfOpenCountup {
                    id: "x".to_string(),
                    block: vec![ast::Stmt {
                        kind: ast::StmtKind::Expr {
                            expr: ast::Expr {
                                kind: ast::ExprKind::Assignment {
                                    var_name: "x".to_string(),
                                    expr: ast::Expr::new_edge(ast::Expr {
                                        kind: ast::ExprKind::Integer { value: 30 }
                                    }),
                                }
                            }
                        }
                    }],
                    from: ast::Expr {
                        kind: ast::ExprKind::Integer { value: 0 }
                    },
                    lessthan: ast::Expr {
                        kind: ast::ExprKind::Integer { value: 10 }
                    }
                }
            },
            n
        );
    }
}
