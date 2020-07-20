use crate::common::{
    ast::{ExNodeId, ExpressionNode, StatementNode},
    token::{Token, TokenKind},
};

use crate::common::compiler::parser::parser_util;

use id_arena::Arena;
use std::sync::{Arc, Mutex};

type StmtArena = Arc<Mutex<Arena<StatementNode>>>;
type ExprArena = Arc<Mutex<Arena<ExpressionNode>>>;

/// expression -> if_expression | assignment
#[allow(clippy::match_single_binding)]
pub fn expression(stmt_arena: StmtArena, expr_arena: ExprArena, module_name: String, tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let head = parser_util::head(&tokens);

    match head.get_kind() {
        TokenKind::IF => if_expression(stmt_arena, expr_arena, module_name, tokens),
        _ => assignment(stmt_arena, expr_arena, module_name, tokens),
    }
}

/// if_expression -> "if" paren_expr block ("else" block)?
fn if_expression(stmt_arena: StmtArena, expr_arena: ExprArena, module_name: String, mut tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let expr_pos = parser_util::current_position(&tokens);
    parser_util::eat_token(&mut tokens);
    let (cond_id, rest_tokens) = paren_expr(stmt_arena.clone(), expr_arena.clone(), module_name.clone(), tokens);

    let (stmts, mut rest_tokens) = parser_util::expect_block(stmt_arena.clone(), expr_arena.clone(), module_name.clone(), rest_tokens);

    let t = parser_util::head(&rest_tokens);
    if t.get_kind() != &TokenKind::ELSE {
        return (
            expr_arena.lock().unwrap().alloc(
                ExpressionNode::new_if(cond_id, stmts, None, expr_pos),
            ),
            rest_tokens
        );
    }

    parser_util::eat_token(&mut rest_tokens);
    let (alter, rest_tokens) = parser_util::expect_block(stmt_arena, expr_arena.clone(), module_name, rest_tokens);

    (
        expr_arena.lock().unwrap().alloc(
            ExpressionNode::new_if(cond_id, stmts, Some(alter), expr_pos),
        ),
        rest_tokens
    )
}

/// assignment -> addition (`=` expression)?
fn assignment(stmt_arena: StmtArena, expr_arena: ExprArena, module_name: String, tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let (lval, mut rest_tokens) = addition(stmt_arena.clone(), expr_arena.clone(), module_name.clone(), tokens);

    let head = parser_util::head(&rest_tokens);

    match head.get_kind() {
        TokenKind::ASSIGN => {
            let assign_pos = head.get_position();
            parser_util::eat_token(&mut rest_tokens);
            let (rval, rest_tokens) = expression(stmt_arena, expr_arena.clone(), module_name, rest_tokens);

            (
                parser_util::alloc_binop_node(
                    expr_arena.lock().unwrap(),
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
fn addition(stmt_arena: StmtArena, expr_arena: ExprArena, module_name: String, tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    parser_util::binary_operation_parser(addition_op, multiplication, stmt_arena, expr_arena, module_name, tokens)
}

/// addition_op -> `+` | `-`
fn addition_op(tokens: Vec<Token>) -> (Option<TokenKind>, Vec<Token>) {
    parser_util::operator_parser(vec![TokenKind::PLUS, TokenKind::MINUS], tokens)
}

/// multiplication -> primary (multiplication_op primary)*
fn multiplication(stmt_arena: StmtArena, expr_arena: ExprArena, module_name: String, tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    parser_util::binary_operation_parser(multiplication_op, prefix, stmt_arena, expr_arena, module_name, tokens)
}

/// multiplication_op -> `*` | `/`
fn multiplication_op(tokens: Vec<Token>) -> (Option<TokenKind>, Vec<Token>) {
    parser_util::operator_parser(vec![TokenKind::ASTERISK, TokenKind::SLASH], tokens)
}

/// prefix -> prefix_op* postfix
fn prefix(stmt_arena: StmtArena, expr_arena: ExprArena, module_name: String, mut tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let head = parser_util::head(&tokens);
    let prefix_pos = head.get_position();

    match head.get_kind() {
        TokenKind::PLUS => {
            parser_util::eat_token(&mut tokens);
            postfix(stmt_arena, expr_arena, module_name, tokens)
        }
        TokenKind::MINUS => {
            parser_util::eat_token(&mut tokens);
            let (value, rest_tokens) = prefix(stmt_arena, expr_arena.clone(), module_name, tokens);
            (
                expr_arena.lock().unwrap().alloc(ExpressionNode::new_prefix_op(&TokenKind::MINUS, value, prefix_pos)),
                rest_tokens,
            )
        }
        TokenKind::AMPERSAND => {
            parser_util::eat_token(&mut tokens);
            let (value, rest_tokens) = prefix(stmt_arena, expr_arena.clone(), module_name, tokens);
            (
                expr_arena.lock().unwrap().alloc(ExpressionNode::new_prefix_op(&TokenKind::AMPERSAND, value, prefix_pos)),
                rest_tokens,
            )
        }
        TokenKind::ASTERISK => {
            parser_util::eat_token(&mut tokens);
            let (value, rest_tokens) = prefix(stmt_arena, expr_arena.clone(), module_name, tokens);
            (
                expr_arena.lock().unwrap().alloc(ExpressionNode::new_prefix_op(&TokenKind::ASTERISK, value, prefix_pos)),
                rest_tokens,
            )
        }
        _ => postfix(stmt_arena, expr_arena, module_name, tokens),
    }
}

/// postfix -> primary (postfix_op postfix)*
fn postfix(stmt_arena: StmtArena, expr_arena: ExprArena, module_name: String, tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let (mut value, mut rest_tokens) = primary(stmt_arena.clone(), expr_arena.clone(), module_name.clone(), tokens);

    loop {
        let head = parser_util::head(&rest_tokens);
        let postfix_pos = head.get_position();

        match head.get_kind() {
            TokenKind::DOT => {
                parser_util::eat_token(&mut rest_tokens);

                let (v, rk) = postfix(stmt_arena.clone(), expr_arena.clone(), module_name.clone(), rest_tokens);
                rest_tokens = rk;

                value =
                    expr_arena.lock().unwrap().alloc(ExpressionNode::new_postfix_op(&TokenKind::DOT, value, v, postfix_pos));
            }
            _ => break,
        }
    }
    (value, rest_tokens)
}

/// primary -> integer_literal | uinteger_literal | "true" | "false" | string_literal | identifier_path | paren_expr
fn primary(stmt_arena: StmtArena, expr_arena: ExprArena, module_name: String, mut tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    let head = parser_util::head(&tokens);
    let pos = head.get_position();

    match head.get_kind() {
        TokenKind::LPAREN => paren_expr(stmt_arena, expr_arena, module_name, tokens),
        TokenKind::INTEGER { value } => {
            parser_util::eat_token(&mut tokens);
            (
                expr_arena.lock().unwrap().alloc(ExpressionNode::new_integer(*value, pos)),
                tokens,
            )
        }
        TokenKind::UNSIGNEDINTEGER { value } => {
            parser_util::eat_token(&mut tokens);
            (
                expr_arena.lock().unwrap().alloc(ExpressionNode::new_uinteger(*value, pos)),
                tokens,
            )
        }
        TokenKind::IDENTIFIER { name: _ } => {
            let (names, tokens) = parser_util::expect_identifier(tokens);
            (
                expr_arena.lock().unwrap().alloc(ExpressionNode::new_identifier(names, pos)),
                tokens,
            )
        }
        TokenKind::STRLIT { contents } => {
            parser_util::eat_token(&mut tokens);
            (
                expr_arena.lock().unwrap().alloc(ExpressionNode::new_string_literal(contents.to_string(), pos)),
                tokens,
            )
        }
        TokenKind::TRUE => {
            parser_util::eat_token(&mut tokens);
            (
                expr_arena.lock().unwrap().alloc(ExpressionNode::new_boolean(true, pos)),
                tokens,
            )
        }
        TokenKind::FALSE => {
            parser_util::eat_token(&mut tokens);
            (
                expr_arena.lock().unwrap().alloc(ExpressionNode::new_boolean(false, pos)),
                tokens,
            )
        }
        _ => panic!("not implemented for `{}` in primary()", head.get_kind()),
    }
}

/// paren_expr -> `(` expression `)`
fn paren_expr(stmt_arena: StmtArena, expr_arena: ExprArena, module_name: String, mut tokens: Vec<Token>) -> (ExNodeId, Vec<Token>) {
    parser_util::eat_token(&mut tokens);

    let (ex_id, mut rest_tokens) = expression(stmt_arena, expr_arena, module_name, tokens);
    parser_util::expect(TokenKind::RPAREN, &mut rest_tokens);

    (ex_id, rest_tokens)
}


#[cfg(test)]
mod expression_tests {
    use super::*;

    fn helper(expr_f: fn(StmtArena, ExprArena, String, Vec<Token>) -> (ExNodeId, Vec<Token>), tokens: Vec<Token>, rest_tokens_number: usize) {
        let (stmt_arena, expr_arena) = new_allocators();
        let (node_id, rest_tokens) = expr_f(
            stmt_arena.clone(),
            expr_arena.clone(),
            Default::default(),
            tokens,
        );

        assert_eq!(rest_tokens_number, rest_tokens.len());

        if let Ok(arena) = expr_arena.lock() {
            let expr_node = arena.get(node_id);

            assert!(expr_node.is_some());
        };
    }

    #[test]
    fn primary_integer_test() {
        let tokens = vec![Token::new_int_literal(30, Default::default())];
        helper(primary, tokens, 0);
    }

    #[test]
    fn paren_expr_test() {
        let tokens = vec![
            Token::new(TokenKind::LPAREN, Default::default()),
            Token::new_int_literal(30, Default::default()),
            Token::new(TokenKind::RPAREN, Default::default()),
        ];
        helper(paren_expr, tokens, 0);
    }

    #[test]
    fn if_expression_test() {
        let tokens = vec![
            Token::new(TokenKind::IF, Default::default()),
            Token::new(TokenKind::LPAREN, Default::default()),
            Token::new_int_literal(30, Default::default()),
            Token::new(TokenKind::RPAREN, Default::default()),
            Token::new(TokenKind::LBRACE, Default::default()),
            Token::new(TokenKind::RETURN, Default::default()),
            Token::new_int_literal(30, Default::default()),
            Token::new(TokenKind::SEMICOLON, Default::default()),
            Token::new(TokenKind::RBRACE, Default::default()),
            Token::new(TokenKind::ELSE, Default::default()),
            Token::new(TokenKind::LBRACE, Default::default()),
            Token::new(TokenKind::RETURN, Default::default()),
            Token::new_int_literal(30, Default::default()),
            Token::new(TokenKind::SEMICOLON, Default::default()),
            Token::new(TokenKind::RBRACE, Default::default()),
        ];
        helper(if_expression, tokens, 0);
    }


    #[test]
    fn primary_identifier_test() {
        let tokens = vec![
            Token::new_identifier("std".to_string(), Default::default()),
            Token::new(TokenKind::DOUBLECOLON, Default::default()),
            Token::new_identifier("os".to_string(), Default::default()),
            Token::new(TokenKind::DOUBLECOLON, Default::default()),
            Token::new_identifier("exit_with".to_string(), Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        helper(primary, tokens, 1);
    }

    #[test]
    fn postfix_test() {
        let tokens = vec![
            Token::new_identifier("x".to_string(), Default::default()),
            Token::new(TokenKind::DOT, Default::default()),
            Token::new_identifier("foo".to_string(), Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        helper(postfix, tokens, 1);
    }

    #[test]
    fn prefix_test() {
        let tokens = vec![
            Token::new(TokenKind::MINUS, Default::default()),
            Token::new_int_literal(30, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        helper(prefix, tokens, 1);
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
        helper(multiplication, tokens, 1);
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
        helper(addition, tokens, 1);
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
        helper(assignment, tokens, 1);
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
