use crate::compiler::common::frontend::{pass::parser::*, types::ast};

use nom::{branch::alt, IResult};
use primitive::{keyword, list_structure, string_literal_str, symbol, Delimiter};

type IResultStmt<'a> = IResult<&'a str, ast::Stmt>;

pub fn statement<'a>() -> impl Fn(&'a str) -> IResultStmt<'a> {
    move |i: &str| alt((expression_statement(), asm_statement()))(i)
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
            list_structure(Delimiter::Bracket, ";", string_literal_str())(rest)?;
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
    fn statement_parser_test_main() {
        let _ = expression_statement_test("u100;", "");
        let _ = asm_statement_test("asm { \"movq $60, %rax\"; \"syscall\" };", "");
        let _ = statement_test("u100;", "");
    }

    fn statement_test<'a>(input: &'a str, rest: &'a str) -> ast::Stmt {
        let result = statement()(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(rest, r);

        n
    }
    fn asm_statement_test<'a>(input: &'a str, rest: &'a str) -> ast::Stmt {
        let result = asm_statement()(input);
        eprintln!("{:?}", result);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();
        assert_eq!(rest, r);

        n
    }
    fn expression_statement_test<'a>(input: &'a str, rest: &'a str) -> ast::Stmt {
        let result = expression_statement()(input);
        assert!(result.is_ok());

        let (r, n) = result.unwrap();

        assert_eq!(rest, r);

        n
    }
}
