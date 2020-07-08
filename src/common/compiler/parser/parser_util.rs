use crate::common::ast::{ExNodeId, ExpressionNode};
use crate::common::position::Position;
use crate::common::token::{Token, TokenKind};
use std::sync::{Arc, Mutex, MutexGuard};

use id_arena::Arena;

type ChildParser = fn(Arc<Mutex<Arena<ExpressionNode>>>, Vec<Token>) -> (ExNodeId, Vec<Token>);
type OperatorParser = fn(Vec<Token>) -> (Option<TokenKind>, Vec<Token>);

pub fn eat_token(tokens: &mut Vec<Token>) {
    tokens.remove(0);
}

pub fn head(tokens: &[Token]) -> Token {
    tokens[0].clone()
}

pub fn current_position(tokens: &[Token]) -> Position {
    tokens[0].get_position()
}

pub fn expect(expected: TokenKind, tokens: &mut Vec<Token>) {
    let h = head(tokens);
    if h.get_kind() != &expected {
        panic!("TODO we must compile error when got difference token in expect()");
    }
    eat_token(tokens);
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
    arena: Arc<Mutex<Arena<ExpressionNode>>>,
    tokens: Vec<Token>,
) -> (ExNodeId, Vec<Token>) {
    let (mut lhs_id, mut rest_tokens) = child_parser(arena.clone(), tokens);

    loop {
        let op_pos = current_position(&rest_tokens);
        let (op, rk) = operator_parser(rest_tokens);
        rest_tokens = rk;
        match op {
            Some(op) => {
                let (rhs_id, rk) = child_parser(arena.clone(), rest_tokens.clone());
                rest_tokens = rk;
                lhs_id = alloc_binop_node(arena.lock().unwrap(), &op, lhs_id, rhs_id, op_pos);
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

/// type -> "Int64" | "Uint64" | "ConstStr"
pub fn expect_type(mut tokens: Vec<Token>) -> (String, Vec<Token>) {
    let type_t = head(&tokens);

    match type_t.get_kind() {
        TokenKind::INT64 => {
            eat_token(&mut tokens);
            ("Int64".to_string(), tokens)
        }
        TokenKind::ASTERISK => {
            eat_token(&mut tokens);
            let (inner_type, mut rest_tokens) = expect_type(tokens);
            eat_token(&mut rest_tokens);
            (format!("*{}", inner_type), rest_tokens)
        }
        TokenKind::IDENTIFIER { name: _ } => {
            let (names, mut rest_tokens) = expect_identifier(tokens);
            eat_token(&mut rest_tokens);
            (names.join("::"), rest_tokens)
        }
        _ => panic!("TODO we must compile error when got difference token in expect_type()"),
    }
}
