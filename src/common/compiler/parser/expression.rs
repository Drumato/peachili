use crate::common::{
    ast::{ExNodeId, ExpressionNode},
    position,
    token::{Token, TokenKind},
};

use crate::common::compiler::parser::parser_util;

use id_arena::Arena;
use std::sync::{Arc, Mutex, MutexGuard};

type ExprArena = Arc<Mutex<Arena<ExpressionNode>>>;

/// expression -> if_expression | assignment
#[allow(clippy::match_single_binding)]
pub fn expression(arena: ExprArena, tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let head = parser_util::head(&tokens);

    match head.get_kind() {
        // TokenKind::IF => if_expression(arena, tokens),
        _ => assignment(arena, tokens),
    }
}

/// assignment -> addition (`=` expression)?
fn assignment(arena: ExprArena, tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let (lval, mut rest_tokens) = addition(arena.clone(), tokens);

    let head = parser_util::head(&rest_tokens);

    match head.get_kind() {
        TokenKind::ASSIGN => {
            let assign_pos = head.get_position();
            parser_util::eat_token(&mut rest_tokens);
            let (rval, rest_tokens) = expression(arena.clone(), rest_tokens);

            (
                parser_util::alloc_binop_node(
                    arena.lock().unwrap(),
                    &TokenKind::ASSIGN,
                    lval,
                    rval,
                    assign_pos,
                ),
                rest_tokens,
            )
        }
        _ => (lval, rest_tokens),
    }
}

/// addition -> multiplication (addition_op multiplication)*
fn addition(arena: ExprArena, tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    parser_util::binary_operation_parser(addition_op, multiplication, arena, tokens)
}

/// addition_op -> `+` | `-`
fn addition_op(tokens: Vec<Token>) -> (Option<TokenKind>, Vec<Token>) {
    parser_util::operator_parser(vec![TokenKind::PLUS, TokenKind::MINUS], tokens)
}

/// multiplication -> primary (multiplication_op primary)*
fn multiplication(arena: ExprArena, tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    parser_util::binary_operation_parser(multiplication_op, prefix, arena, tokens)
}

/// multiplication_op -> `*` | `/`
fn multiplication_op(tokens: Vec<Token>) -> (Option<TokenKind>, Vec<Token>) {
    parser_util::operator_parser(vec![TokenKind::ASTERISK, TokenKind::SLASH], tokens)
}

/// prefix -> prefix_op* postfix
fn prefix(arena: ExprArena, mut tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let head = parser_util::head(&tokens);
    let prefix_pos = head.get_position();

    match head.get_kind() {
        TokenKind::PLUS => {
            parser_util::eat_token(&mut tokens);
            postfix(arena, tokens)
        }
        TokenKind::MINUS => {
            parser_util::eat_token(&mut tokens);
            let (value, rest_tokens) = prefix(arena.clone(), tokens);
            (
                alloc_prefix_op(arena.lock().unwrap(), &TokenKind::MINUS, value, prefix_pos),
                rest_tokens,
            )
        }
        TokenKind::AMPERSAND => {
            parser_util::eat_token(&mut tokens);
            let (value, rest_tokens) = prefix(arena.clone(), tokens);
            (
                alloc_prefix_op(
                    arena.lock().unwrap(),
                    &TokenKind::AMPERSAND,
                    value,
                    prefix_pos,
                ),
                rest_tokens,
            )
        }
        TokenKind::ASTERISK => {
            parser_util::eat_token(&mut tokens);
            let (value, rest_tokens) = prefix(arena.clone(), tokens);
            (
                alloc_prefix_op(
                    arena.lock().unwrap(),
                    &TokenKind::ASTERISK,
                    value,
                    prefix_pos,
                ),
                rest_tokens,
            )
        }
        _ => postfix(arena, tokens),
    }
}

/// postfix -> primary (postfix_op postfix)*
fn postfix(arena: ExprArena, tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let (mut value, mut rest_tokens) = primary(arena.clone(), tokens);

    loop {
        let head = parser_util::head(&rest_tokens);
        let postfix_pos = head.get_position();

        match head.get_kind() {
            TokenKind::DOT => {
                parser_util::eat_token(&mut rest_tokens);
                value =
                    alloc_postfix_op(arena.lock().unwrap(), &TokenKind::DOT, value, postfix_pos);
            }
            _ => break,
        }
    }
    (value, rest_tokens)
}

/// primary -> integer_literal | identifier_path
fn primary(arena: ExprArena, mut tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let head = parser_util::head(&tokens);
    let pos = head.get_position();

    match head.get_kind() {
        TokenKind::INTEGER { value } => {
            parser_util::eat_token(&mut tokens);
            (
                alloc_integer_node(arena.lock().unwrap(), *value, pos),
                tokens,
            )
        }
        TokenKind::IDENTIFIER { name: _ } => {
            let (names, tokens) = expect_identifier(tokens);
            (
                alloc_identifier_node(arena.lock().unwrap(), names, pos),
                tokens,
            )
        }
        _ => panic!("not implemented for `{}` in primary()", head.get_kind()),
    }
}

/// identifier_path -> identifier (`::` identifier)*
fn expect_identifier(mut tokens: Vec<Token>) -> (Vec<String>, Vec<Token>) {
    let head = parser_util::head(&tokens);
    // primary() で識別子であることはチェック済みなのでcopy_name()を読んで良い
    let mut names = vec![head.copy_name()];

    parser_util::eat_token(&mut tokens);
    loop {
        let head = parser_util::head(&tokens);
        match head.get_kind() {
            TokenKind::DOUBLECOLON => {
                parser_util::eat_token(&mut tokens);
                let ident = parser_util::head(&tokens);
                names.push(ident.copy_name());
                parser_util::eat_token(&mut tokens);
            }
            _ => break,
        }
    }

    (names, tokens)
}

/// 前置単項演算ノードのアロケート
fn alloc_prefix_op(
    mut arena: MutexGuard<Arena<ExpressionNode>>,
    operator: &TokenKind,
    value: ExNodeId,
    pos: position::Position,
) -> ExNodeId {
    arena.alloc(ExpressionNode::new_prefix_op(operator, value, pos))
}

/// 後置単項演算ノードのアロケート
fn alloc_postfix_op(
    mut arena: MutexGuard<Arena<ExpressionNode>>,
    operator: &TokenKind,
    value: ExNodeId,
    pos: position::Position,
) -> ExNodeId {
    arena.alloc(ExpressionNode::new_postfix_op(operator, value, pos))
}

/// 整数ノードのアロケート
fn alloc_integer_node(
    mut arena: MutexGuard<Arena<ExpressionNode>>,
    int_value: i64,
    pos: position::Position,
) -> ExNodeId {
    arena.alloc(ExpressionNode::new_integer(int_value, pos))
}

/// 識別子ノードのアロケート
fn alloc_identifier_node(
    mut arena: MutexGuard<Arena<ExpressionNode>>,
    names: Vec<String>,
    pos: position::Position,
) -> ExNodeId {
    arena.alloc(ExpressionNode::new_identifier(names, pos))
}

#[cfg(test)]
mod expression_tests {
    use super::*;
    use crate::common::ast::ExpressionNodeKind;

    #[test]
    fn primary_integer_test() {
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
    fn primary_identifier_test() {
        let arena = new_allocator();
        let (node_id, _rest_tokens) = primary(
            arena.clone(),
            vec![
                Token::new_identifier("std".to_string(), Default::default()),
                Token::new(TokenKind::DOUBLECOLON, Default::default()),
                Token::new_identifier("os".to_string(), Default::default()),
                Token::new(TokenKind::DOUBLECOLON, Default::default()),
                Token::new_identifier("exit_with".to_string(), Default::default()),
                Token::new(TokenKind::EOF, Default::default()),
            ],
        );

        if let Ok(arena) = arena.lock() {
            let ident_node = arena.get(node_id);

            assert!(ident_node.is_some());
            let ident_node = ident_node.unwrap();

            assert_eq!(
                &ExpressionNodeKind::IDENTIFIER {
                    names: vec!["std".to_string(), "os".to_string(), "exit_with".to_string()]
                },
                ident_node.get_kind()
            );
        };
    }

    #[test]
    fn postfix_test() {
        let arena = new_allocator();
        let (node_id, _rest_tokens) = postfix(
            arena.clone(),
            vec![
                Token::new_identifier("x".to_string(), Default::default()),
                Token::new(TokenKind::DOT, Default::default()),
                Token::new_identifier("foo".to_string(), Default::default()),
                Token::new(TokenKind::EOF, Default::default()),
            ],
        );

        if let Ok(arena) = arena.lock() {
            let postfix_node = arena.get(node_id);

            assert!(postfix_node.is_some());
        };
    }

    #[test]
    fn prefix_test() {
        let arena = new_allocator();
        let (node_id, _rest_tokens) = prefix(
            arena.clone(),
            vec![
                Token::new(TokenKind::MINUS, Default::default()),
                Token::new_int_literal(30, Default::default()),
                Token::new(TokenKind::EOF, Default::default()),
            ],
        );

        if let Ok(arena) = arena.lock() {
            let prefix_node = arena.get(node_id);

            assert!(prefix_node.is_some());
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

    #[test]
    fn assignment_test() {
        // `x = y + 2`
        let tokens = vec![
            Token::new_identifier("x".to_string(), Default::default()),
            Token::new(TokenKind::ASSIGN, Default::default()),
            Token::new_identifier("y".to_string(), Default::default()),
            Token::new(TokenKind::PLUS, Default::default()),
            Token::new_int_literal(2, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];

        let arena = new_allocator();
        let (node_id, rest_tokens) = assignment(arena.clone(), tokens);

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
