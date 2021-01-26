use crate::compiler::common::frontend::{pass::parser::*, types::ast};

use nom::{branch::alt, IResult};
use primitive::{keyword, list_structure, string_literal_string, symbol, Delimiter};

type IResultStmt<'a> = IResult<&'a str, ast::Stmt>;

pub fn statement<'a>() -> impl Fn(&'a str) -> IResultStmt<'a> {
    move |i: &str| {
        alt((
            declaration_statement(),
            expression_statement(),
            asm_statement(),
        ))(i)
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
}
