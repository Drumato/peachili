use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::common::position as pos;
use crate::compiler::resource as res;

impl<'a> res::Parser<'a> {
    pub fn expression(&mut self, func_name: &str) -> res::ExpressionNode {
        if self.cur_token_is(&res::TokenKind::IF) {
            return self.if_expression(func_name);
        }
        self.assignment(func_name)
    }

    pub fn if_expression(&mut self, func_name: &str) -> res::ExpressionNode {
        let cur_pos = self.current_position();
        self.progress();

        self.expect(res::TokenKind::LPAREN);

        let condition = self.expression(func_name);
        self.expect(res::TokenKind::RPAREN);

        let body = self.compound_statement(func_name);

        if !self.cur_token_is(&res::TokenKind::ELSE) {
            return res::ExpressionNode::new_if(condition, body, cur_pos);
        }

        self.progress();

        let else_body = self.compound_statement(func_name);

        res::ExpressionNode::new_ifelse(condition, body, else_body, cur_pos)
    }

    pub fn assignment(&mut self, func_name: &str) -> res::ExpressionNode {
        let lval = self.additive(func_name);

        if !self.cur_token_is(&res::TokenKind::ASSIGN) {
            return lval;
        }

        let cur_pos = self.current_position();
        self.progress();

        let rval = self.expression(func_name);

        res::ExpressionNode::new_assign(lval, rval, cur_pos)
    }

    pub fn additive(&mut self, func_name: &str) -> res::ExpressionNode {
        let mut lop = self.multiplicative(func_name);

        loop {
            let cur_pos = self.current_position();

            if self.eat_if_matched(&res::TokenKind::PLUS) {
                lop = res::ExpressionNode::new_add(lop, self.multiplicative(func_name), cur_pos);
            } else if self.eat_if_matched(&res::TokenKind::MINUS) {
                lop = res::ExpressionNode::new_sub(lop, self.multiplicative(func_name), cur_pos);
            } else {
                break;
            }
        }
        lop
    }

    pub fn multiplicative(&mut self, func_name: &str) -> res::ExpressionNode {
        let mut lop = self.unary(func_name);

        loop {
            let cur_pos = self.current_position();

            if self.eat_if_matched(&res::TokenKind::ASTERISK) {
                lop = res::ExpressionNode::new_mul(lop, self.unary(func_name), cur_pos);
            } else if self.eat_if_matched(&res::TokenKind::SLASH) {
                lop = res::ExpressionNode::new_div(lop, self.unary(func_name), cur_pos);
            } else {
                break;
            }
        }

        lop
    }

    pub fn unary(&mut self, func_name: &str) -> res::ExpressionNode {
        let cur_pos = self.current_position();

        if self.eat_if_matched(&res::TokenKind::MINUS) {
            return res::ExpressionNode::new_neg(self.primary(func_name), cur_pos);
        }
        self.eat_if_matched(&res::TokenKind::PLUS);

        self.primary(func_name)
    }

    pub fn primary(&mut self, func_name: &str) -> res::ExpressionNode {
        let cur_kind = self.current_token_kind();

        match cur_kind {
            res::TokenKind::INTEGER(_v) => self.integer_literal(),
            res::TokenKind::UNSIGNEDINTEGER(_v) => self.uint_literal(),
            res::TokenKind::STRLIT(_contents) => self.create_string_symbol(func_name),
            res::TokenKind::IDENTIFIER(_name) => self.identifier(func_name),
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

    pub fn create_string_symbol(&mut self, func_name: &str) -> res::ExpressionNode {
        let (contents, hash, cur_pos) = self.parse_string_literal();
        self.add_string_to(func_name, contents.clone(), hash);

        res::ExpressionNode::new_strlit(contents, hash, cur_pos)
    }

    pub fn parse_string_literal(&mut self) -> (String, u64, pos::Position) {
        let cur_pos = self.current_position();
        let cur_str_contents = self.current_token().copy_strlit_contents();
        self.progress();

        let mut hasher = DefaultHasher::new();

        // 前段のprimary() で文字列リテラルであることを検査しているのでunwrap()してよい．
        let contents = cur_str_contents.unwrap();
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

    pub fn identifier(&mut self, func_name: &str) -> res::ExpressionNode {
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

            args.push(self.expression(func_name));

            self.eat_if_matched(&res::TokenKind::COMMA);
        }

        res::ExpressionNode::new_call(cur_ident, args, cur_pos)
    }
}
