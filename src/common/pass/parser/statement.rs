use crate::common::pass::parser::context::Context;
use crate::common::pass::parser::parser_util;
use crate::common::{
    ast::{ExNodeId, StNodeId, StatementNode, StatementNodeKind},
    token::{Token, TokenKind},
};
use std::collections::BTreeMap;

impl Context {
    /// statement -> return_st | ifret_st | declare_st | countup_st| block_st | asm_st
    pub fn statement(&mut self, tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let head = parser_util::head(&tokens);

        match head.get_kind() {
            TokenKind::MATCH => self.match_statement(tokens),
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

    /// match_statement -> "match" expression `{` pattern* `}`
    fn match_statement(&mut self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        parser_util::eat_token(&mut tokens);

        let (ex_id, mut rest_tokens) = self.expression(tokens);
        parser_util::expect(TokenKind::LBRACE, &mut rest_tokens);

        let mut arms = BTreeMap::new();

        loop {
            if parser_util::consume(TokenKind::RBRACE, &mut rest_tokens) {
                break;
            }

            let (pattern_name, r) = parser_util::expect_identifier(rest_tokens);
            rest_tokens = r;

            parser_util::expect(TokenKind::ARROW, &mut rest_tokens);

            let (stmts, r) = self.expect_block(rest_tokens);
            rest_tokens = r;

            arms.insert(pattern_name.join("::"), stmts);

            parser_util::expect(TokenKind::COMMA, &mut rest_tokens);
        }

        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);
        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::MATCH { expr: ex_id, arms },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    /// return_statement -> "return" expression `;`
    fn return_statement(&mut self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
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
    fn ifret_statement(&mut self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
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
    fn declare_statement(&mut self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        parser_util::eat_token(&mut tokens);

        let (declared_names, mut rest_tokens) = parser_util::expect_identifier(tokens);
        let (type_name, rt) = self.expect_type(rest_tokens);
        rest_tokens = rt;
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::DECLARE {
                    ident_name: declared_names[0].clone(),
                    type_name,
                },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    /// countup_statement -> "countup" identifier "begin" expression "exclude" expression block_statement `;`
    fn countup_statement(&mut self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        parser_util::eat_token(&mut tokens);

        let (ident_names, mut rest_tokens) = parser_util::expect_identifier(tokens);
        let ident_name = ident_names[0].clone();
        parser_util::expect(TokenKind::BEGIN, &mut rest_tokens);

        let (e1_id, mut rest_tokens) = self.expression(rest_tokens);

        parser_util::expect(TokenKind::EXCLUDE, &mut rest_tokens);
        let (e2_id, rest_tokens) = self.expression(rest_tokens);

        let (stmts, mut rest_tokens) = self.expect_block(rest_tokens);
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::COUNTUP {
                    ident_name,
                    begin_ex: e1_id,
                    endpoint_ex: e2_id,
                    body: stmts,
                },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    /// expression_statement -> expression `;`
    fn expression_statement(&mut self, tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
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
    fn asm_statement(&mut self, mut tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        parser_util::eat_token(&mut tokens);

        let (stmts, mut rest_tokens) = self.expect_block(tokens);
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
    fn varinit_statement(&mut self, tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        let (ident, type_name, ex_id, rest_tokens) = self.initialize_statement(tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::VARINIT {
                    ident_name: ident,
                    type_name,
                    expr: ex_id,
                },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    /// const -> "const" identifier type `=` expression `;`
    fn const_statement(&mut self, tokens: Vec<Token>) -> (StNodeId, Vec<Token>) {
        let stmt_pos = parser_util::current_position(&tokens);
        let (ident, type_name, ex_id, rest_tokens) = self.initialize_statement(tokens);

        (
            self.stmt_arena.lock().unwrap().alloc(StatementNode::new(
                StatementNodeKind::CONST {
                    ident_name: ident,
                    type_name,
                    expr: ex_id,
                },
                stmt_pos,
            )),
            rest_tokens,
        )
    }

    fn initialize_statement(
        &mut self,
        mut tokens: Vec<Token>,
    ) -> (String, String, ExNodeId, Vec<Token>) {
        parser_util::eat_token(&mut tokens);

        let (declared_names, mut rest_tokens) = parser_util::expect_identifier(tokens);
        let (type_name, rt) = self.expect_type(rest_tokens);
        rest_tokens = rt;

        parser_util::expect(TokenKind::ASSIGN, &mut rest_tokens);

        let (ex_id, mut rest_tokens) = self.expression(rest_tokens);
        parser_util::expect(TokenKind::SEMICOLON, &mut rest_tokens);

        (declared_names[0].clone(), type_name, ex_id, rest_tokens)
    }
}

#[cfg(test)]
mod statement_tests {
    #[test]
    fn return_statement_test() {}

    #[test]
    fn expr_statement_test() {}

    #[test]
    fn countup_statement_test() {}

    #[test]
    fn asm_statement_test() {}

    #[test]
    fn const_statement_test() {}
}
