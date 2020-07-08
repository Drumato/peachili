use crate::common::compiler::parser::{expression, parser_util};
use crate::common::{
    ast::{ExNodeId, ExpressionNode, StNodeId, StatementNode, StatementNodeKind},
    position,
    token::{Token, TokenKind},
};
use std::sync::{Arc, Mutex, MutexGuard};

use id_arena::Arena;

type StmtArena = Arc<Mutex<Arena<StatementNode>>>;
type ExprArena = Arc<Mutex<Arena<ExpressionNode>>>;

/// statement -> return_statement
pub fn statement(
    stmt_arena: StmtArena,
    expr_arena: ExprArena,
    tokens: Vec<Token>,
) -> (StNodeId, Vec<Token>) {
    let head = parser_util::head(&tokens);

    match head.get_kind() {
        TokenKind::RETURN => return_statement(stmt_arena, expr_arena, tokens),
        TokenKind::IFRET => ifret_statement(stmt_arena, expr_arena, tokens),
        TokenKind::DECLARE => declare_statement(stmt_arena, expr_arena, tokens),
        _ => expression_statement(stmt_arena, expr_arena, tokens),
    }
}

/// return_statement -> "return" expression `;`
fn return_statement(
    stmt_arena: StmtArena,
    expr_arena: ExprArena,
    mut tokens: Vec<Token>,
) -> (StNodeId, Vec<Token>) {
    let stmt_pos = parser_util::current_position(&tokens);
    parser_util::eat_token(&mut tokens);

    let (ex_id, mut rest_tokens) = expression::expression(expr_arena, tokens);
    parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

    (
        alloc_return_statement(stmt_arena.lock().unwrap(), ex_id, stmt_pos),
        rest_tokens,
    )
}

/// ifret_statement -> "ifret" expression `;`
fn ifret_statement(
    stmt_arena: StmtArena,
    expr_arena: ExprArena,
    mut tokens: Vec<Token>,
) -> (StNodeId, Vec<Token>) {
    let stmt_pos = parser_util::current_position(&tokens);
    parser_util::eat_token(&mut tokens);

    let (ex_id, mut rest_tokens) = expression::expression(expr_arena, tokens);
    parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

    (
        alloc_ifret_statement(stmt_arena.lock().unwrap(), ex_id, stmt_pos),
        rest_tokens,
    )
}

/// declare_statement -> "declare" identifier identifier `;`
fn declare_statement(
    stmt_arena: StmtArena,
    _expr_arena: ExprArena,
    mut tokens: Vec<Token>,
) -> (StNodeId, Vec<Token>) {
    let stmt_pos = parser_util::current_position(&tokens);
    parser_util::eat_token(&mut tokens);

    let (declared_names, mut rest_tokens) = parser_util::expect_identifier(tokens);
    let (type_name, rt) = parser_util::expect_type(rest_tokens);
    rest_tokens = rt;
    parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

    (
        alloc_declare_statement(
            stmt_arena.lock().unwrap(),
            declared_names[0].clone(),
            type_name,
            stmt_pos,
        ),
        rest_tokens,
    )
}

/// expression_statement -> expression `;`
fn expression_statement(
    stmt_arena: StmtArena,
    expr_arena: ExprArena,
    tokens: Vec<Token>,
) -> (StNodeId, Vec<Token>) {
    let stmt_pos = parser_util::current_position(&tokens);
    let (ex_id, mut rest_tokens) = expression::expression(expr_arena, tokens);
    parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

    (
        alloc_expression_statement(stmt_arena.lock().unwrap(), ex_id, stmt_pos),
        rest_tokens,
    )
}

fn alloc_return_statement(
    mut arena: MutexGuard<Arena<StatementNode>>,
    ex_id: ExNodeId,
    pos: position::Position,
) -> StNodeId {
    arena.alloc(StatementNode::new(
        StatementNodeKind::RETURNSTMT { expr: ex_id },
        pos,
    ))
}

fn alloc_ifret_statement(
    mut arena: MutexGuard<Arena<StatementNode>>,
    ex_id: ExNodeId,
    pos: position::Position,
) -> StNodeId {
    arena.alloc(StatementNode::new(
        StatementNodeKind::IFRETSTMT { expr: ex_id },
        pos,
    ))
}

fn alloc_declare_statement(
    mut arena: MutexGuard<Arena<StatementNode>>,
    ident: String,
    type_name: String,
    pos: position::Position,
) -> StNodeId {
    arena.alloc(StatementNode::new(
        StatementNodeKind::DECLARESTMT {
            ident_name: ident,
            type_name,
        },
        pos,
    ))
}

fn alloc_expression_statement(
    mut arena: MutexGuard<Arena<StatementNode>>,
    ex_id: ExNodeId,
    pos: position::Position,
) -> StNodeId {
    arena.alloc(StatementNode::new(
        StatementNodeKind::EXPRSTMT { expr: ex_id },
        pos,
    ))
}

#[cfg(test)]
mod statement_tests {
    use super::*;
    use crate::common::ast::*;

    #[test]
    fn return_statement_test() {
        let (stmt_arena, expr_arena) = new_allocators();
        let (node_id, _rest_tokens) = return_statement(
            stmt_arena.clone(),
            expr_arena.clone(),
            vec![
                Token::new(TokenKind::RETURN, Default::default()),
                Token::new(
                    TokenKind::IDENTIFIER {
                        name: "foo".to_string(),
                    },
                    Default::default(),
                ),
                Token::new(TokenKind::SEMICOLON, Default::default()),
                Token::new(TokenKind::EOF, Default::default()),
            ],
        );

        if let Ok(arena) = stmt_arena.lock() {
            let stmt_node = arena.get(node_id);

            assert!(stmt_node.is_some());
        };
    }

    #[test]
    fn expr_statement_test() {
        let (stmt_arena, expr_arena) = new_allocators();
        let (node_id, _rest_tokens) = expression_statement(
            stmt_arena.clone(),
            expr_arena.clone(),
            vec![
                Token::new(
                    TokenKind::IDENTIFIER {
                        name: "foo".to_string(),
                    },
                    Default::default(),
                ),
                Token::new(TokenKind::SEMICOLON, Default::default()),
                Token::new(TokenKind::EOF, Default::default()),
            ],
        );

        if let Ok(arena) = stmt_arena.lock() {
            let stmt_node = arena.get(node_id);

            assert!(stmt_node.is_some());
        };
    }

    #[test]
    fn declare_statement_test() {
        let (stmt_arena, expr_arena) = new_allocators();
        let (node_id, _rest_tokens) = declare_statement(
            stmt_arena.clone(),
            expr_arena.clone(),
            vec![
                Token::new(TokenKind::DECLARE, Default::default()),
                Token::new(
                    TokenKind::IDENTIFIER {
                        name: "foo".to_string(),
                    },
                    Default::default(),
                ),
                Token::new(TokenKind::INT64, Default::default()),
                Token::new(TokenKind::SEMICOLON, Default::default()),
                Token::new(TokenKind::EOF, Default::default()),
            ],
        );

        if let Ok(arena) = stmt_arena.lock() {
            let stmt_node = arena.get(node_id);

            assert!(stmt_node.is_some());
        };
    }

    fn new_allocators() -> (
        Arc<Mutex<Arena<StatementNode>>>,
        Arc<Mutex<Arena<ExpressionNode>>>,
    ) {
        (
            Arc::new(Mutex::new(Arena::new())),
            Arc::new(Mutex::new(Arena::new())),
        )
    }
}
