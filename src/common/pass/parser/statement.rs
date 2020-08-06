use crate::common::pass::parser::{parser_util};
use crate::common::{
    ast::{ExNodeId, StNodeId, StatementNode, StatementNodeKind},
    token::{Token, TokenKind},
};

use crate::common::pass::parser::parse_resource::ParseResource;

impl ParseResource {
    /// statement -> return_st | ifret_st | declare_st | countup_st| block_st | asm_st
    pub fn statement(
        &self,
        tokens: Vec<Token>,
    ) -> (StNodeId, Vec<Token>) {
        let head = parser_util::head(&tokens);

        match head.get_kind() {
            TokenKind::RETURN => self.return_statement(tokens),
            TokenKind::IFRET => self.ifret_statement(tokens),
            TokenKind::DECLARE => self.declare_statement(tokens),
            TokenKind::COUNTUP => self.countup_statement(tokens),
            TokenKind::ASM => self.asm_statement(tokens),
            TokenKind::VARINIT => self.varinit_statement(tokens),
            TokenKind::CONST => self.const_statement(tokens),
            _ => self.expression_statement(tokens),
        }
    }


    /// return_statement -> "return" expression `;`
    fn return_statement(&self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        parser_util::eat_token(&mut tokens);

        let (ex_id, mut rest_tokens) = self.expression(tokens);
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::RETURN { expr: ex_id },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    /// ifret_statement -> "ifret" expression `;`
    fn ifret_statement(&self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        parser_util::eat_token(&mut tokens);

        let (ex_id, mut rest_tokens) = self.expression(tokens);
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::IFRET { expr: ex_id },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    /// declare_statement -> "declare" identifier identifier `;`
    fn declare_statement(&self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        parser_util::eat_token(&mut tokens);

        let (declared_names, mut rest_tokens) = parser_util::expect_identifier(tokens);
        let (type_name, rt) = parser_util::expect_type(self.module_name.clone(), rest_tokens);
        rest_tokens = rt;
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::DECLARE { ident_name: declared_names[0].clone(), type_name },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    /// countup_statement -> "countup" identifier "begin" expression "exclude" expression block_statement `;`
    fn countup_statement(&self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        parser_util::eat_token(&mut tokens);

        let (ident_names, mut rest_tokens) = parser_util::expect_identifier(tokens);
        let ident_name = ident_names[0].clone();
        parser_util::expect(TokenKind::BEGIN, &mut rest_tokens);

        let (e1_id, mut rest_tokens) = self.expression(rest_tokens);

        parser_util::expect(TokenKind::EXCLUDE, &mut rest_tokens);
        let (e2_id, rest_tokens) = self.expression(rest_tokens);

        let (stmts, mut rest_tokens) = parser_util::expect_block(self, rest_tokens);
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::COUNTUP { ident_name, begin_ex: e1_id, endpoint_ex: e2_id, body: stmts },
                stmt_pos,
            )),
            rest_tokens,
        )
    }


    /// expression_statement -> expression `;`
    fn expression_statement(&self, tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        let (ex_id, mut rest_tokens) = self.expression(tokens);
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::EXPR { expr: ex_id },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    /// asm_st -> "asm" block `;`
    fn asm_statement(&self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        parser_util::eat_token(&mut tokens);

        let (stmts, mut rest_tokens) = parser_util::expect_block(self, tokens);
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::ASM { stmts },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    /// varinit -> "varinit" identifier type `=` expression `;`
    fn varinit_statement(&self, tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        let (ident, type_name, ex_id, rest_tokens) = self.initialize_statement(tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::VARINIT { ident_name: ident, type_name, expr: ex_id },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    /// const -> "const" identifier type `=` expression `;`
    fn const_statement(&self, tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        let (ident, type_name, ex_id, rest_tokens) = self.initialize_statement(tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::CONST { ident_name: ident, type_name, expr: ex_id },
                stmt_pos,
            )),
            rest_tokens,
        )
    }


    fn initialize_statement(&self, mut tokens: Vec<Token>) -> (String, String, ExNodeId, Vec<Token>) {
        parser_util::eat_token(&mut tokens);

        let (declared_names, mut rest_tokens) = parser_util::expect_identifier(tokens);
        let (type_name, rt) = parser_util::expect_type(self.module_name.clone(), rest_tokens);
        rest_tokens = rt;

        parser_util::expect(TokenKind::ASSIGN, &mut rest_tokens);

        let (ex_id, mut rest_tokens) = self.expression(rest_tokens);
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (declared_names[0].clone(), type_name, ex_id, rest_tokens)
    }
}


#[cfg(test)]
mod statement_tests {
    use super::*;

    #[test]
    fn return_statement_test() {
        let tokens = vec![
            Token::new(TokenKind::RETURN, Default::default()),
            Token::new(
                TokenKind::IDENTIFIER {
                    name: "foo".to_string(),
                },
                Default::default(),
            ),
            Token::new(TokenKind::SEMICOLON, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        helper(ParseResource::return_statement, tokens, 1);
    }

    #[test]
    fn expr_statement_test() {
        let tokens = vec![
            Token::new(
                TokenKind::IDENTIFIER {
                    name: "foo".to_string(),
                },
                Default::default(),
            ),
            Token::new(TokenKind::SEMICOLON, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        helper(ParseResource::expression_statement, tokens, 1);
    }

    #[test]
    fn countup_statement_test() {
        let tokens = vec![
            Token::new(TokenKind::COUNTUP, Default::default()),
            Token::new(
                TokenKind::IDENTIFIER {
                    name: "foo".to_string(),
                },
                Default::default(),
            ),
            Token::new(TokenKind::BEGIN, Default::default()),
            Token::new_int_literal(0, Default::default()),
            Token::new(TokenKind::EXCLUDE, Default::default()),
            Token::new_int_literal(10, Default::default()),
            Token::new(TokenKind::LBRACE, Default::default()),
            Token::new(TokenKind::RBRACE, Default::default()),
            Token::new(TokenKind::SEMICOLON, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        helper(ParseResource::countup_statement, tokens, 1);
    }

    #[test]
    fn asm_statement_test() {
        let tokens = vec![
            Token::new(TokenKind::ASM, Default::default()),
            Token::new(TokenKind::LBRACE, Default::default()),
            Token::new_string_literal(Default::default(), Default::default()),
            Token::new(TokenKind::SEMICOLON, Default::default()),
            Token::new(TokenKind::RBRACE, Default::default()),
            Token::new(TokenKind::SEMICOLON, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        helper(ParseResource::asm_statement, tokens, 1);
    }


    #[test]
    fn const_statement_test() {
        let tokens = vec![
            Token::new(TokenKind::CONST, Default::default()),
            Token::new(
                TokenKind::IDENTIFIER {
                    name: "foo".to_string(),
                },
                Default::default(),
            ),
            Token::new(TokenKind::INT64, Default::default()),
            Token::new(TokenKind::ASSIGN, Default::default()),
            Token::new_int_literal(3, Default::default()),
            Token::new(TokenKind::SEMICOLON, Default::default()),
            Token::new(TokenKind::EOF, Default::default()),
        ];
        helper(ParseResource::const_statement, tokens, 1);
    }

    fn helper(stmt_f: fn(&ParseResource, Vec<Token>) -> (StNodeId, Vec<Token>), tokens: Vec<Token>, rest_tokens_number: usize) {
        let resources = new_resources();
        let (node_id, rest_tokens) = stmt_f(
            &resources,
            tokens,
        );

        assert_eq!(rest_tokens_number, rest_tokens.len());

        if let Ok(arena) = resources.stmt_arena.lock() {
            let stmt_node = arena.get(node_id);

            assert!(stmt_node.is_some());
        };
    }

    fn new_resources() -> ParseResource {
        ParseResource::new(Default::default())
    }
}
