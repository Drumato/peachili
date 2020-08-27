use crate::common::ast::{ExNodeId, ExpressionNode, StNodeId};
use crate::common::position::Position;
use crate::common::token::{Token, TokenKind};
use std::sync::MutexGuard;

use crate::common::pass::parser::context::Context;
use id_arena::Arena;

type ChildParser = fn(&mut Context, Vec<Token>) -> (ExNodeId, Vec<Token>);
type OperatorParser = fn(&mut Context, Vec<Token>) -> (Option<TokenKind>, Vec<Token>);

impl Context {
    /// type -> "Int64" | "Uint64" | "ConstStr" | "Noreturn" | "Boolean" |`*` type | identifier-path
    pub fn expect_type(&self, mut tokens: Vec<Token>) -> (String, Vec<Token>) {
        let type_t = head(&tokens);

        match type_t.get_kind() {
            TokenKind::INT64 => {
                eat_token(&mut tokens);
                ("Int64".to_string(), tokens)
            }
            TokenKind::UINT64 => {
                eat_token(&mut tokens);
                ("Uint64".to_string(), tokens)
            }
            TokenKind::CONSTSTR => {
                eat_token(&mut tokens);
                ("ConstStr".to_string(), tokens)
            }
            TokenKind::NORETURN => {
                eat_token(&mut tokens);
                ("Noreturn".to_string(), tokens)
            }
            TokenKind::BOOLEAN => {
                eat_token(&mut tokens);
                ("Boolean".to_string(), tokens)
            }

            TokenKind::ASTERISK => {
                eat_token(&mut tokens);
                let (inner_type, rest_tokens) = self.expect_type(tokens);
                (format!("*{}", inner_type), rest_tokens)
            }
            TokenKind::IDENTIFIER { name: _ } => {
                let (names, rest_tokens) = expect_identifier(tokens);
                (
                    format!("{}::{}", self.module_name, names.join("::")),
                    rest_tokens,
                )
            }
            _ => panic!("TODO we must compile error when got difference token in expect_type()"),
        }
    }

    /// block -> `{` statement* `}`
    pub fn expect_block(&mut self, mut tokens: Vec<Token>) -> (Vec<StNodeId>, Vec<Token>) {
        eat_token(&mut tokens);

        let mut stmts: Vec<StNodeId> = Vec::new();

        loop {
            let h = head(&tokens);

            if h.get_kind() == &TokenKind::RBRACE {
                eat_token(&mut tokens);
                break;
            }

            let (st_id, rt) = self.statement(tokens);
            stmts.push(st_id);
            tokens = rt;
        }

        (stmts, tokens)
    }
}

pub fn eat_token(tokens: &mut Vec<Token>) {
    if tokens.is_empty() {
        panic!("cannot remove top of tokens because Vec<Token> is empty");
    }
    tokens.remove(0);
}

pub fn head(tokens: &[Token]) -> Token {
    if tokens.is_empty() {
        return Token::new(TokenKind::EOF, Default::default());
    }
    tokens[0].clone()
}

pub fn current_position(tokens: &[Token]) -> Position {
    if tokens.is_empty() {
        return Default::default();
    }
    tokens[0].get_position()
}

pub fn expect(expected: TokenKind, tokens: &mut Vec<Token>) {
    let h = head(tokens);
    if h.get_kind() != &expected {
        panic!("expected => {:?}, got => {:?}", expected, h.get_kind());
    }
    eat_token(tokens);
}

pub fn consume(expected: TokenKind, tokens: &mut Vec<Token>) -> bool {
    let h = head(tokens);
    if h.get_kind() != &expected {
        return false;
    }
    eat_token(tokens);

    true
}

pub fn operator_parser(
    operators: Vec<TokenKind>,
    mut tokens: Vec<Token>,
) -> (Option<TokenKind>, Vec<Token>) {
    let head = head(&tokens);

    for tk in operators.iter() {
        if tk == head.get_kind() {
            eat_token(&mut tokens);

            return (Some(tk.clone()), tokens);
        }
    }

    (None, tokens)
}

pub fn binary_operation_parser(
    operator_parser: OperatorParser,
    child_parser: ChildParser,
    ctxt: &mut Context,
    tokens: Vec<Token>,
) -> (ExNodeId, Vec<Token>) {
    let (mut lhs_id, mut rest_tokens) = child_parser(ctxt, tokens);

    loop {
        let op_pos = current_position(&rest_tokens);
        let (op, rk) = operator_parser(ctxt, rest_tokens);
        rest_tokens = rk;
        match op {
            Some(op) => {
                let (rhs_id, rk) = child_parser(ctxt, rest_tokens.clone());
                rest_tokens = rk;
                lhs_id =
                    alloc_binop_node(ctxt.expr_arena.lock().unwrap(), &op, lhs_id, rhs_id, op_pos);
            }
            None => break,
        }
    }

    (lhs_id, rest_tokens)
}

/// 二項演算ノードのアロケート
pub fn alloc_binop_node(
    mut arena: MutexGuard<Arena<ExpressionNode>>,
    k: &TokenKind,
    lhs: ExNodeId,
    rhs: ExNodeId,
    pos: Position,
) -> ExNodeId {
    arena.alloc(ExpressionNode::new_binop(k, lhs, rhs, pos))
}

/// identifier_path -> identifier (`::` identifier)*
pub fn expect_identifier(mut tokens: Vec<Token>) -> (Vec<String>, Vec<Token>) {
    let base = head(&tokens);
    // primary() で識別子であることはチェック済みなのでcopy_name()を読んで良い
    let mut names = vec![base.copy_name()];

    eat_token(&mut tokens);
    loop {
        let next = head(&tokens);
        match next.get_kind() {
            TokenKind::DOUBLECOLON => {
                eat_token(&mut tokens);
                let ident = head(&tokens);
                names.push(ident.copy_name());
                eat_token(&mut tokens);
            }
            _ => break,
        }
    }

    (names, tokens)
}

#[cfg(test)]
mod parser_util_tests {
    use super::*;

    #[test]
    fn expect_identifier_test() {
        let tokens = vec![
            Token::new_identifier("std".to_string(), Default::default()),
            Token::new(TokenKind::DOUBLECOLON, Default::default()),
            Token::new_identifier("os".to_string(), Default::default()),
            Token::new(TokenKind::DOUBLECOLON, Default::default()),
            Token::new_identifier("FileDescriptor".to_string(), Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];

        let (names, rest_tokens) = expect_identifier(tokens);
        assert_eq!(1, rest_tokens.len());
        assert_eq!(
            vec![
                "std".to_string(),
                "os".to_string(),
                "FileDescriptor".to_string()
            ],
            names
        );
    }

    #[test]
    fn expect_type_test() {
    }

    #[test]
    fn expect_block_test() {
    }
}
