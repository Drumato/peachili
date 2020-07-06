use crate::common::{
    ast::{ExNodeId, ExpressionNode},
    position,
    token::{Token, TokenKind},
};

use crate::common::compiler::parser::parser_util;

use id_arena::Arena;
use std::sync::{Arc, Mutex, MutexGuard};

/// addition -> multiplication (addition_op multiplication)*
pub fn addition(
    arena: Arc<Mutex<Arena<ExpressionNode>>>,
    tokens: Vec<Token>,
) -> (ExNodeId, Vec<Token>) {
    parser_util::binary_operation_parser(addition_op, multiplication, arena, tokens)
}

/// addition_op -> `+` | `-`
pub fn addition_op(tokens: Vec<Token>) -> (Option<TokenKind>, Vec<Token>) {
    parser_util::operator_parser(vec![TokenKind::PLUS, TokenKind::MINUS], tokens)
}

/// multiplication -> primary (multiplication_op primary)*
fn multiplication(
    arena: Arc<Mutex<Arena<ExpressionNode>>>,
    tokens: Vec<Token>,
) -> (ExNodeId, Vec<Token>) {
    parser_util::binary_operation_parser(multiplication_op, primary, arena, tokens)
}

/// multiplication_op -> `*` | `/`
pub fn multiplication_op(tokens: Vec<Token>) -> (Option<TokenKind>, Vec<Token>) {
    parser_util::operator_parser(vec![TokenKind::ASTERISK, TokenKind::SLASH], tokens)
}

/// primary -> integer_literal
fn primary(
    arena: Arc<Mutex<Arena<ExpressionNode>>>,
    mut tokens: Vec<Token>,
) -> (ExNodeId, Vec<Token>) {
    let head = tokens[0].clone();
    let pos = head.get_position();

    let n = match head.get_kind() {
        TokenKind::INTEGER { value } => {
            parser_util::eat_token(&mut tokens);
            alloc_integer_node(arena.lock().unwrap(), *value, pos)
        }
        _ => panic!("not implemented for `{}` in primary()", head.get_kind()),
    };

    (n, tokens)
}

/// 整数ノードのアロケート
fn alloc_integer_node(
    mut arena: MutexGuard<Arena<ExpressionNode>>,
    int_value: i64,
    pos: position::Position,
) -> ExNodeId {
    arena.alloc(ExpressionNode::new_integer(int_value, pos))
}

#[cfg(test)]
mod expression_tests {
    use super::*;
    use crate::common::ast::ExpressionNodeKind;

    #[test]
    fn primary_test() {
        let arena = new_allocator();
        let (node_id, rest_tokens) = primary(
            arena.clone(),
            vec![Token::new_int_literal(30, Default::default())],
        );

        assert!(rest_tokens.is_empty());

        if let Ok(arena) = arena.lock() {
            let integer_node = arena.get(node_id);

            assert!(integer_node.is_some());
            let integer_node = integer_node.unwrap();

            assert_eq!(
                &ExpressionNodeKind::INTEGER { value: 30 },
                integer_node.get_kind()
            );
        };
    }

    #[test]
    fn multiplication_test() {
        // `30 * 50`
        let tokens = vec![
            Token::new_int_literal(30, Default::default()),
            Token::new(TokenKind::ASTERISK, Default::default()),
            Token::new_int_literal(50, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];

        let arena = new_allocator();
        let (node_id, rest_tokens) = multiplication(arena.clone(), tokens);

        assert_eq!(1, rest_tokens.len());

        if let Ok(arena) = arena.lock() {
            let binop_node = arena.get(node_id);
            assert!(binop_node.is_some());
        };
    }

    #[test]
    fn addition_test() {
        // `1 * 2 + 3 * 4`
        let tokens = vec![
            Token::new_int_literal(1, Default::default()),
            Token::new(TokenKind::ASTERISK, Default::default()),
            Token::new_int_literal(2, Default::default()),
            Token::new(TokenKind::PLUS, Default::default()),
            Token::new_int_literal(3, Default::default()),
            Token::new(TokenKind::ASTERISK, Default::default()),
            Token::new_int_literal(4, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];

        let arena = new_allocator();
        let (node_id, rest_tokens) = addition(arena.clone(), tokens);

        assert_eq!(1, rest_tokens.len());

        if let Ok(arena) = arena.lock() {
            let binop_node = arena.get(node_id);
            assert!(binop_node.is_some());
        };
    }

    fn new_allocator() -> Arc<Mutex<Arena<ExpressionNode>>> {
        Arc::new(Mutex::new(Arena::new()))
    }
}
