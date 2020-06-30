use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::common::position as pos;
use crate::compiler::general::resource as res;

impl<'a> res::Parser<'a> {
    pub fn expression(&mut self, func_name_id: res::PStringId) -> res::ExpressionNode {
        if self.cur_token_is(&res::TokenKind::IF) {
            return self.if_expression(func_name_id);
        }
        self.assignment(func_name_id)
    }

    pub fn if_expression(&mut self, func_name_id: res::PStringId) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        self.progress();

        self.expect(res::TokenKind::LPAREN);

        let condition = self.expression(func_name_id);
        self.expect(res::TokenKind::RPAREN);

        let body = self.compound_statement(func_name_id);

        if !self.cur_token_is(&res::TokenKind::ELSE) {
            return res::ExpressionNode::new_if(condition, body, cur_pos);
        }

        self.progress();

        let else_body = self.compound_statement(func_name_id);

        res::ExpressionNode::new_ifelse(condition, body, else_body, cur_pos)
    }

    pub fn assignment(&mut self, func_name_id: res::PStringId) -> res::ExpressionNode {
        let lval = self.additive(func_name_id);

        if !self.cur_token_is(&res::TokenKind::ASSIGN) {
            return lval;
        }

        let cur_pos = self.current_position();
        self.progress();

        let rval = self.expression(func_name_id);

        res::ExpressionNode::new_binary_expr("=", lval, rval, cur_pos)
    }

    pub fn additive(&mut self, func_name_id: res::PStringId) -> res::ExpressionNode {
        let mut lop = self.multiplicative(func_name_id);

        loop {
            let cur_pos = self.current_position();

            if self.eat_if_matched(&res::TokenKind::PLUS) {
                lop = res::ExpressionNode::new_binary_expr("+", lop, self.multiplicative(func_name_id), cur_pos);
            } else if self.eat_if_matched(&res::TokenKind::MINUS) {
                lop = res::ExpressionNode::new_binary_expr("-", lop, self.multiplicative(func_name_id), cur_pos);
            } else {
                break;
            }
        }
        lop
    }

    pub fn multiplicative(&mut self, func_name_id: res::PStringId) -> res::ExpressionNode {
        let mut lop = self.unary(func_name_id);

        loop {
            let cur_pos = self.current_position();

            if self.eat_if_matched(&res::TokenKind::ASTERISK) {
                lop = res::ExpressionNode::new_binary_expr("*", lop, self.unary(func_name_id), cur_pos);
            } else if self.eat_if_matched(&res::TokenKind::SLASH) {
                lop = res::ExpressionNode::new_binary_expr("/", lop, self.unary(func_name_id), cur_pos);
            } else {
                break;
            }
        }

        lop
    }

    pub fn unary(&mut self, func_name_id: res::PStringId) -> res::ExpressionNode {
        let cur_kind = self.current_token_kind();
        let cur_pos = self.current_position();

        match cur_kind {
            res::TokenKind::PLUS => {
                self.progress();
                self.postfix(func_name_id)
            }
            res::TokenKind::MINUS => {
                self.progress();
                res::ExpressionNode::new_unary_expr("-", self.postfix(func_name_id), cur_pos)
            }
            res::TokenKind::ASTERISK => {
                self.progress();
                res::ExpressionNode::new_unary_expr("*", self.postfix(func_name_id), cur_pos)
            }
            res::TokenKind::AMPERSAND => {
                self.progress();
                res::ExpressionNode::new_unary_expr("&", self.postfix(func_name_id), cur_pos)
            }

            _ => self.postfix(func_name_id),
        }
    }

    pub fn postfix(&mut self, func_name_id: res::PStringId) -> res::ExpressionNode {
        let mut n = self.primary(func_name_id);

        loop {
            let cur_pos = self.current_position();

            if self.eat_if_matched(&res::TokenKind::DOT) {
                let member_id = self.expect_name().unwrap();
                n = res::ExpressionNode::new_member(n, member_id, cur_pos);
            } else {
                break;
            }
        }

        n
    }

    pub fn primary(&mut self, func_name_id: res::PStringId) -> res::ExpressionNode {
        let cur_kind = self.current_token_kind();

        match cur_kind {
            res::TokenKind::INTEGER(_v) => self.integer_literal(),
            res::TokenKind::UNSIGNEDINTEGER(_v) => self.uint_literal(),
            res::TokenKind::STRLIT(_contents) => self.create_string_symbol(func_name_id),
            res::TokenKind::IDENTIFIER(_name) => self.identifier(func_name_id),
            res::TokenKind::TRUE => self.boolean_literal(true),
            res::TokenKind::FALSE => self.boolean_literal(false),
            _ => panic!("unexpected {} in primary", cur_kind.to_str()),
        }
    }

    pub fn integer_literal(&mut self) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        let cur_int_value = self.current_token().get_int_value();
        self.progress();

        // primary() で整数リテラルであることを検査しているのでunwrap()してよい．
        res::ExpressionNode::new_intlit(cur_int_value.unwrap(), cur_pos)
    }
    pub fn uint_literal(&mut self) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        let cur_uint_value = self.current_token().get_uint_value();
        self.progress();

        // primary() で整数リテラルであることを検査しているのでunwrap()してよい．
        res::ExpressionNode::new_uintlit(cur_uint_value.unwrap(), cur_pos)
    }

    pub fn create_string_symbol(&mut self, func_name_id: res::PStringId) -> res::ExpressionNode {
        let (contents_id, hash, cur_pos) = self.parse_string_literal();
        self.add_string_to(func_name_id, contents_id, hash);

        res::ExpressionNode::new_strlit(contents_id, hash, cur_pos)
    }

    pub fn parse_string_literal(&mut self) -> (res::PStringId, u64, pos::Position) {
        let cur_pos = self.current_position();
        let str_contents_id = self.current_token().get_str_id();
        self.progress();

        let mut hasher = DefaultHasher::new();

        // 前段のprimary() で文字列リテラルであることを検査しているのでunwrap()してよい．
        let contents = str_contents_id.unwrap();
        contents.hash(&mut hasher);

        (contents, hasher.finish(), cur_pos)
    }

    pub fn boolean_literal(&mut self, boolean: bool) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        self.progress();

        if boolean {
            res::ExpressionNode::new_true(cur_pos)
        } else {
            res::ExpressionNode::new_false(cur_pos)
        }
    }

    pub fn identifier(&mut self, func_name_id: res::PStringId) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        let cur_ident = self.expect_identifier();

        if !self.eat_if_matched(&res::TokenKind::LPAREN) {
            return res::ExpressionNode::new_ident(cur_ident, cur_pos);
        }

        let mut args: Vec<res::ExpressionNode> = Vec::new();

        loop {
            if self.eat_if_matched(&res::TokenKind::RPAREN) {
                break;
            }

            args.push(self.expression(func_name_id));

            self.eat_if_matched(&res::TokenKind::COMMA);
        }

        res::ExpressionNode::new_call(cur_ident, args, cur_pos)
    }
}
