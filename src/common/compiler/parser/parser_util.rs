use crate::common::ast::{ExNodeId, ExpressionNode,StatementNode, StNodeId};
use crate::common::position::Position;
use crate::common::token::{Token, TokenKind};
use std::sync::{Arc, Mutex, MutexGuard};
use crate::common::compiler::parser::statement;

use id_arena::Arena;

type StmtArena = Arc<Mutex<Arena<StatementNode>>>;
type ExprArena = Arc<Mutex<Arena<ExpressionNode>>>;

type ChildParser = fn(StmtArena, ExprArena, Vec<Token>) -> (ExNodeId, Vec<Token>);
type OperatorParser = fn(Vec<Token>) -> (Option<TokenKind>, Vec<Token>);

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
        panic!("TODO we must compile error when got difference token in expect()");
    }
    eat_token(tokens);
}

pub fn consume(expected: TokenKind, tokens: &mut Vec<Token>) -> bool{
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
    stmt_arena: StmtArena,
    expr_arena: ExprArena,
    tokens: Vec<Token>,
) -> (ExNodeId, Vec<Token>) {
    let (mut lhs_id, mut rest_tokens) = child_parser(stmt_arena.clone(), expr_arena.clone(), tokens);

    loop {
        let op_pos = current_position(&rest_tokens);
        let (op, rk) = operator_parser(rest_tokens);
        rest_tokens = rk;
        match op {
            Some(op) => {
                let (rhs_id, rk) = child_parser(stmt_arena.clone(), expr_arena.clone(), rest_tokens.clone());
                rest_tokens = rk;
                lhs_id = alloc_binop_node(expr_arena.lock().unwrap(), &op, lhs_id, rhs_id, op_pos);
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

/// type -> "Int64" | "Uint64" | "ConstStr" | "Noreturn" | "Boolean" |`*` type | identifier-path
pub fn expect_type(mut tokens: Vec<Token>) -> (String, Vec<Token>) {
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
            let (inner_type, rest_tokens) = expect_type(tokens);
            (format!("*{}", inner_type), rest_tokens)
        }
        TokenKind::IDENTIFIER { name: _ } => {
            let (names, rest_tokens) = expect_identifier(tokens);
            (names.join("::"), rest_tokens)
        }
        _ => panic!("TODO we must compile error when got difference token in expect_type()"),
    }
}


/// block -> `{` statement* `}`
pub fn expect_block(stmt_arena: StmtArena, expr_arena: ExprArena, mut tokens: Vec<Token>) -> (Vec<StNodeId>, Vec<Token>) {
    eat_token(&mut tokens);

    let mut stmts: Vec<StNodeId> = Vec::new();

    loop {
        let h = head(&tokens);

        if h.get_kind() == &TokenKind::RBRACE {
            eat_token(&mut tokens);
            break;
        }

        let (st_id, rt) = statement::statement(stmt_arena.clone(), expr_arena.clone(), tokens);
        stmts.push(st_id);
        tokens = rt;
    }

    (stmts, tokens)
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
        assert_eq!(vec!["std".to_string(), "os".to_string(), "FileDescriptor".to_string()], names);
    }

    #[test]
    fn expect_type_test() {
        let tokens = vec![
            Token::new(TokenKind::ASTERISK, Default::default()),
            Token::new(TokenKind::ASTERISK, Default::default()),
            Token::new(TokenKind::ASTERISK, Default::default()),
            Token::new_identifier("std".to_string(), Default::default()),
            Token::new(TokenKind::DOUBLECOLON, Default::default()),
            Token::new_identifier("os".to_string(), Default::default()),
            Token::new(TokenKind::DOUBLECOLON, Default::default()),
            Token::new_identifier("FileDescriptor".to_string(), Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];

        let (type_string, rest_tokens) = expect_type(tokens);
        assert_eq!(1, rest_tokens.len());
        assert_eq!("***std::os::FileDescriptor", type_string);
    }

    #[test]
    fn expect_block_test() {
        let tokens = vec![
            Token::new(TokenKind::LBRACE, Default::default()),
            Token::new(TokenKind::RBRACE, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];

        let (stmt_arena, expr_arena) =  new_allocators();
        let (stmts, rest_tokens) = expect_block(stmt_arena, expr_arena, tokens);
        assert_eq!(1, rest_tokens.len());
        assert_eq!(0, stmts.len());
    }

    fn new_allocators() -> (
        StmtArena,
        ExprArena,
    ) {
        (
            Arc::new(Mutex::new(Arena::new())),
            Arc::new(Mutex::new(Arena::new())),
        )
    }
}